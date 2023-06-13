use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};
use unic_langid::LanguageIdentifier;

use super::use_init_i18n::UseInitI18Data;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Language {
    id: LanguageIdentifier,
    texts: Text,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Text {
    Value(String),
    Texts(HashMap<String, Text>),
}

impl Default for Text {
    fn default() -> Self {
        Self::Texts(HashMap::default())
    }
}

impl Text {
    fn query(&self, steps: &mut Vec<&str>) -> Option<String> {
        match self {
            Text::Texts(texts) => {
                if steps.is_empty() {
                    return None;
                }

                let current_path = steps.join(".");

                // Try quering the next step in this list
                let this_step = steps.remove(0);
                let deep = texts.get(this_step)?;
                let res = deep.query(steps);

                // If not found try quering by the whole remaining path as if it was the ID
                if res.is_none() {
                    let res_text = texts.get(&current_path);
                    if let Some(res_text) = res_text {
                        return res_text.query(steps);
                    }
                }
                res
            }
            Text::Value(value) => Some(value.to_owned()),
        }
    }
}

impl FromStr for Language {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).map_err(|_| ())
    }
}

impl Language {
    pub fn get_text(&self, path: &str, params: HashMap<&str, String>) -> Option<String> {
        let mut steps = path.split('.').collect::<Vec<&str>>();

        let mut text = self.texts.query(&mut steps).unwrap_or_default();

        for (name, value) in params {
            text = text.replacen(&format!("{{{name}}}"), &value.to_string(), 1);
        }
        Some(text)
    }
}

#[derive(Clone, Copy)]
pub struct UseI18<'a> {
    pub selected_language: &'a UseSharedState<LanguageIdentifier>,
    pub data: &'a UseSharedState<UseInitI18Data>,
}

impl<'a> UseI18<'a> {
    pub fn translate_with_params(&self, id: &str, params: HashMap<&str, String>) -> String {
        let i18n_data = self.data.read();

        // Try searching in the selected language
        for language in i18n_data.languages.iter() {
            if language.id == *self.selected_language.read() {
                return language.get_text(id, params).unwrap_or_default();
            }
        }

        // Otherwise find in the fallback language
        for language in i18n_data.languages.iter() {
            if language.id == i18n_data.fallback_language {
                return language.get_text(id, params).unwrap_or_default();
            }
        }

        // Return the ID as there is no alternative
        id.to_string()
    }

    pub fn translate(&self, id: &str) -> String {
        self.translate_with_params(id, HashMap::default())
    }

    pub fn set_language(&self, id: LanguageIdentifier) {
        *self.selected_language.write() = id;
    }
}

pub fn use_i18(cx: &ScopeState) -> UseI18 {
    let selected_language = use_shared_state::<LanguageIdentifier>(cx).unwrap();
    let data = use_shared_state::<UseInitI18Data>(cx).unwrap();

    UseI18 {
        selected_language,
        data,
    }
}
