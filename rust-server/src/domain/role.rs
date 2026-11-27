//! Account roles and the registration codes that grant them.
//!
//! Roles are a property of the account (`users.role`), not of the session, so
//! signing in again never promotes or demotes anybody.

use serde::{Deserialize, Serialize};

/// Password that creates an administrator account.
pub const ADMIN_CODE: &str = "19112009";
/// Password that creates a cheater account.
pub const CHEAT_CODE: &str = "1778";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    #[default]
    Player,
    Cheater,
    Admin,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Player => "player",
            Role::Cheater => "cheater",
            Role::Admin => "admin",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "player" => Some(Role::Player),
            "cheater" => Some(Role::Cheater),
            "admin" => Some(Role::Admin),
            _ => None,
        }
    }

    /// Role granted by a password typed at registration.
    ///
    /// `None` means "an ordinary password": the account is a plain player.
    pub fn from_registration_password(password: &str) -> Option<Role> {
        match password {
            ADMIN_CODE => Some(Role::Admin),
            CHEAT_CODE => Some(Role::Cheater),
            _ => None,
        }
    }

    pub fn is_admin(self) -> bool {
        matches!(self, Role::Admin)
    }

    pub fn is_cheater(self) -> bool {
        matches!(self, Role::Cheater)
    }

    /// Can open the admin panel and act on other accounts.
    pub fn can_administer(self) -> bool {
        self.is_admin()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_grant_roles() {
        assert_eq!(Role::from_registration_password("19112009"), Some(Role::Admin));
        assert_eq!(Role::from_registration_password("1778"), Some(Role::Cheater));
        assert_eq!(Role::from_registration_password("ordinary-passphrase"), None);
    }

    #[test]
    fn wire_names_round_trip() {
        for role in [Role::Player, Role::Cheater, Role::Admin] {
            assert_eq!(Role::parse(role.as_str()), Some(role));
        }
    }

    #[test]
    fn only_admin_can_administer() {
        assert!(Role::Admin.can_administer());
        assert!(!Role::Cheater.can_administer());
        assert!(!Role::Player.can_administer());
    }
}
