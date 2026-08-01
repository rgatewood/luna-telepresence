/// Embedded default templates using compile-time inclusion.

const TEMPLATES: &[(&str, &str)] = &[
    ("general", include_str!("../../../templates/general.json")),
    ("requirements", include_str!("../../../templates/requirements.json")),
    ("proposal_opportunity_intake", include_str!("../../../templates/proposal_opportunity_intake.json")),
    ("solicitation_intake", include_str!("../../../templates/solicitation_intake.json")),
    ("proposal_presentation", include_str!("../../../templates/proposal_presentation.json")),
    ("candidate_interview", include_str!("../../../templates/candidate_interview.json")),
    ("verbatim", include_str!("../../../templates/verbatim.json")),
    ("status", include_str!("../../../templates/status.json")),
    ("internal_one_to_one", include_str!("../../../templates/internal_one_to_one.json")),
    ("client_stakeholder_one_to_one", include_str!("../../../templates/client_stakeholder_one_to_one.json")),
    ("agenda_led_workshop_or_interview", include_str!("../../../templates/agenda_led_workshop_or_interview.json")),
    ("capabilities_introduction_call", include_str!("../../../templates/capabilities_introduction_call.json")),
];

pub fn get_builtin_templates() -> Vec<(&'static str, &'static str)> {
    TEMPLATES.to_vec()
}

pub fn get_builtin_template(id: &str) -> Option<&'static str> {
    TEMPLATES.iter().find(|(key, _)| *key == id).map(|(_, value)| *value)
}

pub fn list_builtin_template_ids() -> Vec<&'static str> {
    TEMPLATES.iter().map(|(key, _)| *key).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_twelve_builtin_templates_are_valid_json() {
        assert_eq!(TEMPLATES.len(), 12);
        for (id, content) in get_builtin_templates() {
            assert!(serde_json::from_str::<serde_json::Value>(content).is_ok(), "invalid template: {id}");
        }
    }

    #[test]
    fn builtins_are_addressable_by_id() {
        for id in list_builtin_template_ids() {
            assert!(get_builtin_template(id).is_some());
        }
        assert!(get_builtin_template("nonexistent").is_none());
    }
}
