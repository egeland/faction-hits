use crate::api::FactionAttack;
use crate::state::State;

#[derive(Debug, Clone)]
pub struct NewHit {
    pub attacker_name: String,
    pub attacker_id: i64,
    pub defender_name: String,
    pub defender_id: i64,
    pub result: String,
    pub respect: f64,
    pub timestamp: i64,
}

impl From<FactionAttack> for NewHit {
    fn from(attack: FactionAttack) -> Self {
        Self {
            attacker_name: attack.attacker_name,
            attacker_id: attack.attacker_id,
            defender_name: attack.defender_name,
            defender_id: attack.defender_id,
            result: attack.result,
            respect: attack.respect,
            timestamp: attack.timestamp,
        }
    }
}

pub fn filter_new_hits(attacks: &[FactionAttack], state: &State) -> Vec<NewHit> {
    attacks
        .iter()
        .filter(|attack| attack.timestamp > state.last_check_timestamp && !attack.stealth)
        .map(|attack| NewHit::from(attack.clone()))
        .collect()
}

pub fn get_latest_timestamp(attacks: &[FactionAttack]) -> Option<i64> {
    attacks.iter().map(|a| a.timestamp).max()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::FactionAttack;

    fn create_attack(id: i64, timestamp: i64, stealth: bool) -> FactionAttack {
        FactionAttack {
            id,
            attacker_id: 1,
            attacker_name: "Attacker".to_string(),
            defender_id: 2,
            defender_name: "Defender".to_string(),
            result: "Lost".to_string(),
            stealth,
            respect: 1.0,
            timestamp,
        }
    }

    #[test]
    fn test_filter_new_hits_excludes_anonymous() {
        let attacks = vec![
            create_attack(1, 100, true),
            create_attack(2, 100, true),
            create_attack(3, 100, true),
        ];
        let state = State {
            last_check_timestamp: 50,
            faction_id: None,
        };

        let new_hits = filter_new_hits(&attacks, &state);
        assert!(new_hits.is_empty());
    }

    #[test]
    fn test_filter_new_hits_includes_non_anonymous() {
        let attacks = vec![create_attack(1, 100, false), create_attack(2, 100, false)];
        let state = State {
            last_check_timestamp: 50,
            faction_id: None,
        };

        let new_hits = filter_new_hits(&attacks, &state);
        assert_eq!(new_hits.len(), 2);
    }

    #[test]
    fn test_filter_new_hits_respects_timestamp() {
        let attacks = vec![
            create_attack(1, 30, false),
            create_attack(2, 60, false),
            create_attack(3, 90, false),
        ];
        let state = State {
            last_check_timestamp: 50,
            faction_id: None,
        };

        let new_hits = filter_new_hits(&attacks, &state);
        assert_eq!(new_hits.len(), 2);
        assert!(new_hits.iter().all(|h| h.timestamp > 50));
    }

    #[test]
    fn test_filter_new_hits_empty_attacks() {
        let attacks: Vec<FactionAttack> = vec![];
        let state = State {
            last_check_timestamp: 50,
            faction_id: None,
        };

        let new_hits = filter_new_hits(&attacks, &state);
        assert!(new_hits.is_empty());
    }

    #[test]
    fn test_filter_new_hits_mixed_stealth() {
        let attacks = vec![
            create_attack(1, 100, true),
            create_attack(2, 100, false),
            create_attack(3, 100, true),
            create_attack(4, 100, false),
        ];
        let state = State {
            last_check_timestamp: 50,
            faction_id: None,
        };

        let new_hits = filter_new_hits(&attacks, &state);
        assert_eq!(new_hits.len(), 2);
    }

    #[test]
    fn test_filter_new_hits_converts_correctly() {
        let attack = FactionAttack {
            id: 123,
            attacker_id: 456,
            attacker_name: "TestAttacker".to_string(),
            defender_id: 789,
            defender_name: "TestDefender".to_string(),
            result: "Lost".to_string(),
            stealth: false,
            respect: 2.5,
            timestamp: 999,
        };
        let state = State {
            last_check_timestamp: 0,
            faction_id: None,
        };

        let new_hits = filter_new_hits(&[attack], &state);
        assert_eq!(new_hits.len(), 1);

        let hit = &new_hits[0];
        assert_eq!(hit.attacker_id, 456);
        assert_eq!(hit.attacker_name, "TestAttacker");
        assert_eq!(hit.defender_id, 789);
        assert_eq!(hit.defender_name, "TestDefender");
        assert_eq!(hit.result, "Lost");
        assert_eq!(hit.respect, 2.5);
        assert_eq!(hit.timestamp, 999);
    }

    #[test]
    fn test_get_latest_timestamp() {
        let attacks = vec![
            create_attack(1, 30, false),
            create_attack(2, 100, false),
            create_attack(3, 60, false),
        ];
        assert_eq!(get_latest_timestamp(&attacks), Some(100));
    }

    #[test]
    fn test_get_latest_timestamp_empty() {
        let attacks: Vec<FactionAttack> = vec![];
        assert_eq!(get_latest_timestamp(&attacks), None);
    }

    #[test]
    fn test_get_latest_timestamp_single() {
        let attacks = vec![create_attack(1, 42, false)];
        assert_eq!(get_latest_timestamp(&attacks), Some(42));
    }

    #[test]
    fn test_new_hit_from_attack() {
        let attack = FactionAttack {
            id: 1,
            attacker_id: 100,
            attacker_name: "Alice".to_string(),
            defender_id: 200,
            defender_name: "Bob".to_string(),
            result: "Hospitalized".to_string(),
            stealth: false,
            respect: 1.75,
            timestamp: 1234567890,
        };

        let hit: NewHit = attack.into();
        assert_eq!(hit.attacker_id, 100);
        assert_eq!(hit.attacker_name, "Alice");
        assert_eq!(hit.defender_id, 200);
        assert_eq!(hit.defender_name, "Bob");
        assert_eq!(hit.result, "Hospitalized");
        assert_eq!(hit.respect, 1.75);
        assert_eq!(hit.timestamp, 1234567890);
    }
}
