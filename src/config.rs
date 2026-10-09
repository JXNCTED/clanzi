use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use serde_json::{Map, Value};
use std::fmt;

pub const LIMIT: usize = 64 * 1024;
pub const CONTROLS: [&str; 6] = [
    "key_top",
    "key_middle",
    "key_bottom",
    "knob_press",
    "knob_right",
    "knob_left",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Modifier {
    Ctrl,
    Shift,
    Alt,
    Super,
}

impl Modifier {
    pub fn usage(self) -> u16 {
        match self {
            Self::Ctrl => 224,
            Self::Shift => 225,
            Self::Alt => 226,
            Self::Super => 227,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Action {
    #[default]
    None,
    Key {
        usage: u16,
        modifiers: Vec<Modifier>,
    },
    // 兼容上一版配置；所有按钮现在都直接跟随物理按下/松开。
    Hold {
        usage: u16,
        modifiers: Vec<Modifier>,
    },
}

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
        enum WireAction {
            None {},
            Key {
                usage: u16,
                modifiers: Vec<Modifier>,
            },
            Hold {
                usage: u16,
                modifiers: Vec<Modifier>,
            },
        }
        Ok(match WireAction::deserialize(deserializer)? {
            WireAction::None {} => Self::None,
            WireAction::Key { usage, modifiers } => Self::Key { usage, modifiers },
            WireAction::Hold { usage, modifiers } => Self::Hold { usage, modifiers },
        })
    }
}

impl Action {
    pub fn keys(&self) -> Vec<u16> {
        match self {
            Self::None => vec![],
            Self::Key { usage, modifiers } | Self::Hold { usage, modifiers } => modifiers
                .iter()
                .map(|m| m.usage())
                .chain([*usage])
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bindings {
    pub key_top: Action,
    pub key_middle: Action,
    pub key_bottom: Action,
    pub knob_press: Action,
    pub knob_right: Action,
    pub knob_left: Action,
}

impl Bindings {
    pub fn get(&self, index: u8) -> Option<&Action> {
        [
            &self.key_top,
            &self.key_middle,
            &self.key_bottom,
            &self.knob_press,
            &self.knob_right,
            &self.knob_left,
        ]
        .get(index as usize)
        .copied()
    }
    fn actions_mut(&mut self) -> [&mut Action; 6] {
        [
            &mut self.key_top,
            &mut self.key_middle,
            &mut self.key_bottom,
            &mut self.knob_press,
            &mut self.knob_right,
            &mut self.knob_left,
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Keymap {
    pub format: String,
    pub version: u32,
    pub device: String,
    pub bindings: Bindings,
}

impl Default for Keymap {
    fn default() -> Self {
        Self {
            format: "olanzi-cross-platform-keymap".into(),
            version: 1,
            device: "AU05".into(),
            bindings: Bindings::default(),
        }
    }
}

pub fn allowed_usage(usage: u16) -> bool {
    matches!(usage, 4..=49 | 51..=56 | 58..=69 | 73..=82)
}

impl Keymap {
    pub fn validate(mut self) -> Result<Self, String> {
        if self.format != "olanzi-cross-platform-keymap"
            || !matches!(self.version, 1 | 2)
            || self.device != "AU05"
        {
            return Err("unsupported format or version".into());
        }
        for (index, action) in self.bindings.actions_mut().into_iter().enumerate() {
            if matches!(action, Action::Hold { .. }) && (self.version != 2 || index >= 4) {
                return Err("hold requires version 2 and a button control".into());
            }
            if let Action::Key { usage, modifiers } | Action::Hold { usage, modifiers } = action {
                if !allowed_usage(*usage) {
                    return Err("unsupported keyboard usage".into());
                }
                modifiers.sort();
                if modifiers.len() > 4 || modifiers.windows(2).any(|w| w[0] == w[1]) {
                    return Err("duplicate modifiers".into());
                }
            }
        }
        Ok(self)
    }
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > LIMIT {
            return Err("configuration exceeds 64 KiB".into());
        }
        let value: UniqueValue = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        serde_json::from_value::<Self>(value.0)
            .map_err(|e| e.to_string())?
            .validate()
    }
    pub fn bytes(&self) -> Vec<u8> {
        serde_json::to_vec_pretty(self).expect("serializable keymap")
    }
}

// 先拒绝所有层级的重复字段，避免枚举反序列化或 Value 静默覆盖它们。
struct UniqueValue(Value);
impl<'de> Deserialize<'de> for UniqueValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct UniqueVisitor;
        impl<'de> Visitor<'de> for UniqueVisitor {
            type Value = UniqueValue;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("JSON with unique properties")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
                Ok(UniqueValue(v.into()))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(UniqueValue(v.into()))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(UniqueValue(v.into()))
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
                Ok(UniqueValue(Value::from(v)))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(UniqueValue(v.into()))
            }
            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(UniqueValue(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(value) = a.next_element::<UniqueValue>()? {
                    values.push(value.0);
                }
                Ok(UniqueValue(Value::Array(values)))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
                let mut values = Map::new();
                while let Some(key) = a.next_key::<String>()? {
                    if values.contains_key(&key) {
                        return Err(de::Error::custom("duplicate property"));
                    }
                    values.insert(key, a.next_value::<UniqueValue>()?.0);
                }
                Ok(UniqueValue(Value::Object(values)))
            }
        }
        deserializer.deserialize_any(UniqueVisitor)
    }
}
