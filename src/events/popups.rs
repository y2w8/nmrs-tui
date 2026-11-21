use crossterm::event::{KeyCode, KeyEvent};
use nmrs::WifiSecurity;

use crate::{
    action::{Action, ConnectRequest},
    app::{App, Focus},
    ui::popup::Popups,
};

pub fn handle(app: &mut App, key: KeyEvent) {
    let Focus::Popup(popup) = &mut app.focus else {
        return;
    };

    match popup {
        Popups::Password(input, network) => match key.code {
            // Connect
            KeyCode::Enter => {
                app.action.send(Action::Connect(Box::new(ConnectRequest {
                    ssid: network.ssid.clone(),
                    interface: None,
                    credentials: WifiSecurity::WpaPsk {
                        psk: input.value.clone(),
                    },
                })));

                popup.close(&app.action, app.last_focus.clone());
            }

            // Hide the password
            KeyCode::Tab => {
                input.hidden = !input.hidden;
            }
            KeyCode::Char(to_insert) => input.enter_char(to_insert),
            KeyCode::Backspace => input.delete_char(),
            KeyCode::Left => input.move_cursor_left(),
            KeyCode::Right => input.move_cursor_right(),
            KeyCode::Esc => {
                popup.close(&app.action, app.last_focus.clone());
            }
            _ => {}
        },
    }
}
