use std::{error::Error, io, mem, ops::ControlFlow};
mod scanner;

use scanner::{Scanner, Token, TokenKind};
mod interpreter;
use interpreter::{ast::Visitor, env::Env, parser::Parser, *};
use ratatui::{prelude::*, symbols::border, widgets::*};

mod util;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use util::*;

struct Completion {
    index: usize,
    completions: Vec<String>,
}

struct App {
    tokens: Vec<Token>,
    input: String,
    /// Position of cursor in the editor area.
    cursor_position: usize,
    message: String,
    history: Vec<String>,
    history_index: usize,
    completion: Option<Completion>,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            cursor_position: 0,
            tokens: Vec::new(),
            message: String::new(),
            history: Vec::new(),
            history_index: 0,
            completion: None,
        }
    }
}

impl App {
    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }
    fn incr_history(&mut self) {
        if self.history_index < self.history.len() {
            self.history_index += 1;
            self.input = self
                .history
                .get(self.history_index)
                .cloned()
                .unwrap_or(String::new());
            self.cursor_position = self.input.len()
        }
    }
    fn decr_history(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.input = self.history[self.history_index].clone();
            self.cursor_position = self.input.len()
        }
    }
    fn enter_char(&mut self, new_char: char) {
        self.input.insert(self.cursor_position, new_char);

        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_position != 0;
        if is_not_cursor_leftmost {
            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.input.chars().skip(current_index);

            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.len())
    }

    fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }

    fn update_completions(&mut self, env: &Env) {
        self.completion = get_ident_at_end(&self.input[..self.cursor_position]).and_then(|s| {
            let completions: Vec<_> = env.search(s).map(|(name, _)| name.to_string()).collect();
            (!completions.is_empty()).then(|| Completion {
                index: 0,
                completions,
            })
        })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let stdout = io::stdout();

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Inline(8),
        },
    )?;

    // create app and run it
    let app = App::default();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;

    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn color_tokens(buf: &mut Buffer, tokens: &[Token], x: u16, y: u16) {
    for (i, t) in tokens.iter().enumerate() {
        let peek = tokens.get(i + 1).map(|Token { kind, .. }| kind);
        buf.set_style(
            Rect::new(x + t.start as u16, y, (t.end - t.start) as u16, 1),
            match t.kind {
                TokenKind::Plus
                | TokenKind::Slash
                | TokenKind::Minus
                | TokenKind::Star
                | TokenKind::Exp => Style::default().fg(Color::LightCyan),
                TokenKind::Number => Style::default().fg(Color::Magenta),
                TokenKind::Indentifier if peek == Some(&TokenKind::LParen) => {
                    Style::default().fg(Color::Blue)
                }
                TokenKind::Indentifier => Style::default().fg(Color::Red),
                TokenKind::LParen | TokenKind::RParen => Style::default().fg(Color::DarkGray),
                _ => Style::default(),
            },
        );
    }
}
fn handle_key_event<B: Backend>(
    key: KeyEvent,
    terminal: &mut Terminal<B>,
    app: &mut App,
    interpreter: &mut Interpreter,
) -> io::Result<ControlFlow<()>> {
    if key.kind == KeyEventKind::Press {
        if let Some(comp) = &mut app.completion {
            match key.code {
                KeyCode::Enter | KeyCode::Tab => {
                    if let Some(r) = get_ident_range(&app.input, app.cursor_position) {
                        app.input
                            .replace_range(r.clone(), &comp.completions[comp.index]);
                        app.cursor_position = r.start + comp.completions[comp.index].len()
                    }
                }
                KeyCode::Down => comp.index = (comp.index + 1).min(comp.completions.len() - 1),
                KeyCode::Up => comp.index = comp.index.saturating_sub(1),
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Up => app.decr_history(),
                KeyCode::Down => app.incr_history(),
                _ => {}
            }
        };
        match key.code {
            KeyCode::Enter => {
                let res = Parser::new(&app.tokens, &app.input)
                    .parse()
                    .and_then(|e| interpreter.visit_stmt_owned(e));
                if let Ok(res) = res {
                    interpreter.last_ans = res.clone();
                    terminal.insert_before(3, |b| {
                        Paragraph::new(vec![
                            Line::raw(""),
                            Line::raw(&app.input),
                            Line::from(vec![
                                Span::raw("= "),
                                Span::styled(
                                    disp_num(&res, DISPLAY_DIGITS).unwrap(),
                                    Style::default().fg(Color::Red),
                                ),
                            ]),
                        ])
                        .render(b.area, b);
                        color_tokens(b, &app.tokens, 0, 1);
                    })?;

                    app.history.push(mem::take(&mut app.input));
                    app.history_index = app.history.len();
                    app.reset_cursor()
                }
            }
            KeyCode::Char(to_insert) if to_insert.is_ascii() => {
                app.enter_char(to_insert);
                app.update_completions(&interpreter.env);
            }
            KeyCode::Backspace => {
                app.delete_char();
                app.update_completions(&interpreter.env);
            }
            KeyCode::Left => {
                app.move_cursor_left();
            }
            KeyCode::Right => {
                app.move_cursor_right();
            }
            KeyCode::Esc => {
                return Ok(ControlFlow::Break(()));
            }
            _ => {}
        }
    };
    Ok(ControlFlow::Continue(()))
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let mut interpreter = Interpreter::new();
    loop {
        app.tokens = Scanner::new(&app.input).scan_tokens().unwrap();
        interpreter.save_assignments = false;
        let res = Parser::new(&app.tokens, &app.input)
            .parse()
            .and_then(|e| interpreter.visit_stmt_owned(e));
        app.message = match res {
            Ok(n) => format!("Current result {}", disp_num(&n, DISPLAY_DIGITS).unwrap()),
            Err(e) => format!("{}", e),
        };

        interpreter.save_assignments = true;
        terminal.draw(|f| ui(f, &app, &interpreter))?;

        match event::read()? {
            Event::Key(key) => {
                if handle_key_event(key, terminal, &mut app, &mut interpreter)?.is_break() {
                    break Ok(());
                }
            }
            Event::Resize(_, _) => terminal.autoresize()?,
            _ => (),
        }
    }
}

fn ui(f: &mut Frame, app: &App, interpreter: &Interpreter) {
    let vertical = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Min(0),
    ]);

    let [mut msg_area, input_area, mut completion_area] = vertical.areas(f.size());
    if !app.message.is_empty() {
        let msg = Paragraph::new(format!("{}", app.message)).block(
            Block::default()
                .borders(Borders::ALL.difference(Borders::BOTTOM))
                .border_set(border::ONE_EIGHTH_WIDE),
        );
        msg_area.width = app.message.len() as u16 + 2;

        f.render_widget(msg, msg_area);
    }
    let input = Paragraph::new(app.input.as_str()).style(Style::default());

    f.render_widget(input, input_area);
    let buf = f.buffer_mut();
    color_tokens(buf, &app.tokens, input_area.x, input_area.y);

    f.set_cursor(input_area.x + app.cursor_position as u16, input_area.y);

    if let Some(comp) = &app.completion {
        let completions_list = List::new(
            comp.completions
                .iter()
                .take(completion_area.height as usize)
                .map(|s| s.as_str()),
        )
        .highlight_style(Style::default().on_dark_gray());

        completion_area.height = completion_area.height.min(completions_list.len() as u16);
        completion_area.width = 32;
        f.render_stateful_widget(
            completions_list,
            completion_area,
            &mut ListState::default().with_selected(Some(comp.index)),
        );
    }
}
