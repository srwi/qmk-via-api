use crate::keycodes::{Keycode, KeycodeCategory};

#[cfg(feature = "python")]
use pyo3::prelude::*;

/// Standard QMK keycode ranges (protocol version 12).
pub mod ranges {
    use std::ops::Range;

    pub const QK_BASIC: Range<u16> = 0x0000..0x0100;
    pub const QK_MODS: Range<u16> = 0x0100..0x2000;
    pub const QK_MOD_TAP: Range<u16> = 0x2000..0x4000;
    pub const QK_LAYER_TAP: Range<u16> = 0x4000..0x5000;
    pub const QK_LAYER_MOD: Range<u16> = 0x5000..0x5200;
    pub const QK_TO: Range<u16> = 0x5200..0x5220;
    pub const QK_MOMENTARY: Range<u16> = 0x5220..0x5240;
    pub const QK_DEF_LAYER: Range<u16> = 0x5240..0x5260;
    pub const QK_TOGGLE_LAYER: Range<u16> = 0x5260..0x5280;
    pub const QK_ONE_SHOT_LAYER: Range<u16> = 0x5280..0x52A0;
    pub const QK_ONE_SHOT_MOD: Range<u16> = 0x52A0..0x52C0;
    pub const QK_LAYER_TAP_TOGGLE: Range<u16> = 0x52C0..0x52E0;
    pub const QK_TAP_DANCE: Range<u16> = 0x5700..0x5800;
    pub const QK_MACRO: Range<u16> = 0x7700..0x7780;
    pub const QK_KB: Range<u16> = 0x7E00..0x7E40;
    pub const QK_USER: Range<u16> = 0x7E40..0x8000;
}

/// A 5-bit modifier mask used in QMK quantum keycodes (bits 0-3 for Ctrl, Shift, Alt, Gui; bit 4 for Right hand).
#[cfg_attr(feature = "python", pyclass(from_py_object, eq))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct QmkModMask(pub u8);

impl QmkModMask {
    pub const LCTL: u8 = 0x01;
    pub const LSFT: u8 = 0x02;
    pub const LALT: u8 = 0x04;
    pub const LGUI: u8 = 0x08;
    pub const RIGHT_HAND: u8 = 0x10;

    pub const RCTL: u8 = Self::LCTL | Self::RIGHT_HAND;
    pub const RSFT: u8 = Self::LSFT | Self::RIGHT_HAND;
    pub const RALT: u8 = Self::LALT | Self::RIGHT_HAND;
    pub const RGUI: u8 = Self::LGUI | Self::RIGHT_HAND;

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn from_bits(bits: u8) -> Self {
        Self(bits & 0x1F)
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl QmkModMask {
    #[new]
    #[pyo3(signature = (bits=0))]
    pub fn new(bits: u8) -> Self {
        Self::from_bits(bits)
    }
}

#[cfg_attr(feature = "python", pymethods)]
impl QmkModMask {
    pub const fn bits(&self) -> u8 {
        self.0
    }

    pub const fn is_empty(&self) -> bool {
        (self.0 & 0x0F) == 0
    }

    pub const fn has_ctrl(&self) -> bool {
        (self.0 & Self::LCTL) != 0
    }

    pub const fn has_shift(&self) -> bool {
        (self.0 & Self::LSFT) != 0
    }

    pub const fn has_alt(&self) -> bool {
        (self.0 & Self::LALT) != 0
    }

    pub const fn has_gui(&self) -> bool {
        (self.0 & Self::LGUI) != 0
    }

    pub const fn is_right(&self) -> bool {
        (self.0 & Self::RIGHT_HAND) != 0
    }

    pub fn with_right(&self, right: bool) -> Self {
        if right {
            Self(self.0 | Self::RIGHT_HAND)
        } else {
            Self(self.0 & !Self::RIGHT_HAND)
        }
    }
}

/// QMK layer switching operations.
#[cfg_attr(feature = "python", pyclass(from_py_object, eq, eq_int))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QmkLayerOp {
    Momentary,
    Toggle,
    To,
    OneShot,
    TapToggle,
    Default,
}

impl QmkLayerOp {
    pub const ALL: [QmkLayerOp; 6] = [
        QmkLayerOp::Momentary,
        QmkLayerOp::Toggle,
        QmkLayerOp::To,
        QmkLayerOp::OneShot,
        QmkLayerOp::TapToggle,
        QmkLayerOp::Default,
    ];

    pub const fn range(&self) -> std::ops::Range<u16> {
        match self {
            Self::To => ranges::QK_TO,
            Self::Momentary => ranges::QK_MOMENTARY,
            Self::Default => ranges::QK_DEF_LAYER,
            Self::Toggle => ranges::QK_TOGGLE_LAYER,
            Self::OneShot => ranges::QK_ONE_SHOT_LAYER,
            Self::TapToggle => ranges::QK_LAYER_TAP_TOGGLE,
        }
    }
}

#[cfg_attr(feature = "python", pymethods)]
impl QmkLayerOp {
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Momentary => "MO",
            Self::Toggle => "TG",
            Self::To => "TO",
            Self::OneShot => "OSL",
            Self::TapToggle => "TT",
            Self::Default => "DF",
        }
    }

    pub const fn encode(&self, layer: u8) -> Option<u16> {
        if layer > 31 {
            return None;
        }
        let base = match self {
            Self::To => ranges::QK_TO.start,
            Self::Momentary => ranges::QK_MOMENTARY.start,
            Self::Default => ranges::QK_DEF_LAYER.start,
            Self::Toggle => ranges::QK_TOGGLE_LAYER.start,
            Self::OneShot => ranges::QK_ONE_SHOT_LAYER.start,
            Self::TapToggle => ranges::QK_LAYER_TAP_TOGGLE.start,
        };
        Some(base + layer as u16)
    }
}

/// Encodes a modifier combination: `(mods << 8) | keycode`. Returns `None` if mods is empty.
#[cfg_attr(feature = "python", pyfunction)]
pub fn encode_mod_combo(mods: QmkModMask, keycode: u8) -> Option<u16> {
    if mods.is_empty() {
        return None;
    }
    let code = ((mods.bits() as u16) << 8) | (keycode as u16);
    ranges::QK_MODS.contains(&code).then_some(code)
}

/// Encodes a Mod-Tap keycode. Returns `None` if mods is empty.
#[cfg_attr(feature = "python", pyfunction)]
pub fn encode_mod_tap(mods: QmkModMask, keycode: u8) -> Option<u16> {
    if mods.is_empty() {
        return None;
    }
    Some(ranges::QK_MOD_TAP.start + ((mods.bits() as u16) << 8) + (keycode as u16))
}

/// Encodes a Layer-Tap keycode. Returns `None` if layer > 15.
#[cfg_attr(feature = "python", pyfunction)]
pub fn encode_layer_tap(layer: u8, keycode: u8) -> Option<u16> {
    if layer > 15 {
        return None;
    }
    Some(ranges::QK_LAYER_TAP.start + ((layer as u16) << 8) + (keycode as u16))
}

/// Encodes a Layer-Mod keycode. Returns `None` if layer > 15 or mods is empty.
#[cfg_attr(feature = "python", pyfunction)]
pub fn encode_layer_mod(layer: u8, mods: QmkModMask) -> Option<u16> {
    if layer > 15 || mods.is_empty() {
        return None;
    }
    Some(ranges::QK_LAYER_MOD.start + ((layer as u16) << 5) + (mods.bits() as u16))
}

/// Encodes a One-Shot Modifier keycode. Returns `None` if mods is empty.
#[cfg_attr(feature = "python", pyfunction)]
pub fn encode_one_shot_mod(mods: QmkModMask) -> Option<u16> {
    if mods.is_empty() {
        return None;
    }
    Some(ranges::QK_ONE_SHOT_MOD.start + (mods.bits() as u16))
}

/// A structured representation of a QMK keycode.
#[cfg_attr(feature = "python", pyclass(from_py_object, eq))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum QmkKeycode {
    /// A recognized named keycode (basic keycode or fixed function).
    Keycode(Keycode),
    /// Modifier combo: `(mods << 8) | keycode` (`0x0100..0x2000`).
    ModCombo { mods: QmkModMask, keycode: u8 },
    /// Mod-Tap keycode: `0x2000 + (mods << 8) + keycode` (`0x2000..0x4000`).
    ModTap { mods: QmkModMask, keycode: u8 },
    /// Layer-Tap keycode: `0x4000 + (layer << 8) + keycode` (`0x4000..0x5000`).
    LayerTap { layer: u8, keycode: u8 },
    /// Layer-Mod keycode: `0x5000 + (layer << 5) + mods` (`0x5000..0x5200`).
    LayerMod { layer: u8, mods: QmkModMask },
    /// Layer switch operations (`0x5200..0x52E0` excluding `0x52A0..0x52C0`).
    LayerOp { op: QmkLayerOp, layer: u8 },
    /// One-shot modifier (`0x52A0..0x52C0`).
    OneShotMod(QmkModMask),
    /// Tap dance (`0x5700..0x5800`).
    TapDance(u8),
    /// Macro (`0x7700..0x7780`).
    Macro(u8),
    /// Keyboard-specific custom keycode (`0x7E00..0x7E40`).
    CustomKb(u8),
    /// User custom keycode (`0x7E40..0x8000`).
    CustomUser(u16),
    /// Unrecognized raw keycode value.
    Raw(u16),
}

impl QmkKeycode {
    /// Decodes a 16-bit QMK keycode into a structured `QmkKeycode`.
    pub fn from_u16(code: u16) -> Self {
        use ranges::*;

        if QK_MODS.contains(&code) {
            let mods = QmkModMask::from_bits(((code >> 8) & 0x1F) as u8);
            let keycode = (code & 0xFF) as u8;
            return Self::ModCombo { mods, keycode };
        }

        if QK_MOD_TAP.contains(&code) {
            let rem = code - QK_MOD_TAP.start;
            let mods = QmkModMask::from_bits(((rem >> 8) & 0x1F) as u8);
            let keycode = (rem & 0xFF) as u8;
            return Self::ModTap { mods, keycode };
        }

        if QK_LAYER_TAP.contains(&code) {
            let rem = code - QK_LAYER_TAP.start;
            let layer = ((rem >> 8) & 0x0F) as u8;
            let keycode = (rem & 0xFF) as u8;
            return Self::LayerTap { layer, keycode };
        }

        if QK_LAYER_MOD.contains(&code) {
            let rem = code - QK_LAYER_MOD.start;
            let layer = ((rem >> 5) & 0x0F) as u8;
            let mods = QmkModMask::from_bits((rem & 0x1F) as u8);
            return Self::LayerMod { layer, mods };
        }

        if QK_TO.contains(&code) {
            return Self::LayerOp {
                op: QmkLayerOp::To,
                layer: (code - QK_TO.start) as u8,
            };
        }
        if QK_MOMENTARY.contains(&code) {
            return Self::LayerOp {
                op: QmkLayerOp::Momentary,
                layer: (code - QK_MOMENTARY.start) as u8,
            };
        }
        if QK_DEF_LAYER.contains(&code) {
            return Self::LayerOp {
                op: QmkLayerOp::Default,
                layer: (code - QK_DEF_LAYER.start) as u8,
            };
        }
        if QK_TOGGLE_LAYER.contains(&code) {
            return Self::LayerOp {
                op: QmkLayerOp::Toggle,
                layer: (code - QK_TOGGLE_LAYER.start) as u8,
            };
        }
        if QK_ONE_SHOT_LAYER.contains(&code) {
            return Self::LayerOp {
                op: QmkLayerOp::OneShot,
                layer: (code - QK_ONE_SHOT_LAYER.start) as u8,
            };
        }
        if QK_ONE_SHOT_MOD.contains(&code) {
            return Self::OneShotMod(QmkModMask::from_bits((code - QK_ONE_SHOT_MOD.start) as u8));
        }
        if QK_LAYER_TAP_TOGGLE.contains(&code) {
            return Self::LayerOp {
                op: QmkLayerOp::TapToggle,
                layer: (code - QK_LAYER_TAP_TOGGLE.start) as u8,
            };
        }

        if QK_TAP_DANCE.contains(&code) {
            return Self::TapDance((code - QK_TAP_DANCE.start) as u8);
        }
        if QK_MACRO.contains(&code) {
            return Self::Macro((code - QK_MACRO.start) as u8);
        }
        if QK_KB.contains(&code) {
            return Self::CustomKb((code - QK_KB.start) as u8);
        }
        if QK_USER.contains(&code) {
            return Self::CustomUser(code - QK_USER.start);
        }

        if let Ok(kc) = Keycode::try_from(code) {
            Self::Keycode(kc)
        } else {
            Self::Raw(code)
        }
    }

    /// Encodes a modifier combination: `(mods << 8) | keycode`. Returns `None` if mods is empty.
    pub fn encode_mod_combo(mods: QmkModMask, keycode: u8) -> Option<u16> {
        encode_mod_combo(mods, keycode)
    }

    /// Encodes a Mod-Tap keycode. Returns `None` if mods is empty.
    pub fn encode_mod_tap(mods: QmkModMask, keycode: u8) -> Option<u16> {
        encode_mod_tap(mods, keycode)
    }

    /// Encodes a Layer-Tap keycode. Returns `None` if layer > 15.
    pub fn encode_layer_tap(layer: u8, keycode: u8) -> Option<u16> {
        encode_layer_tap(layer, keycode)
    }

    /// Encodes a Layer-Mod keycode. Returns `None` if layer > 15 or mods is empty.
    pub fn encode_layer_mod(layer: u8, mods: QmkModMask) -> Option<u16> {
        encode_layer_mod(layer, mods)
    }

    /// Encodes a One-Shot Modifier keycode. Returns `None` if mods is empty.
    pub fn encode_one_shot_mod(mods: QmkModMask) -> Option<u16> {
        encode_one_shot_mod(mods)
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl QmkKeycode {
    #[new]
    pub fn new(code: u16) -> Self {
        Self::from_u16(code)
    }
}

#[cfg_attr(feature = "python", pymethods)]
impl QmkKeycode {
    /// Encodes this `QmkKeycode` into a 16-bit QMK keycode.
    pub fn to_u16(&self) -> u16 {
        match self {
            Self::Keycode(kc) => *kc as u16,
            Self::ModCombo { mods, keycode } => ((mods.bits() as u16) << 8) | (*keycode as u16),
            Self::ModTap { mods, keycode } => {
                ranges::QK_MOD_TAP.start + ((mods.bits() as u16) << 8) + (*keycode as u16)
            }
            Self::LayerTap { layer, keycode } => {
                ranges::QK_LAYER_TAP.start + ((*layer as u16) << 8) + (*keycode as u16)
            }
            Self::LayerMod { layer, mods } => {
                ranges::QK_LAYER_MOD.start + ((*layer as u16) << 5) + (mods.bits() as u16)
            }
            Self::LayerOp { op, layer } => op.encode(*layer).unwrap_or(0),
            Self::OneShotMod(mods) => ranges::QK_ONE_SHOT_MOD.start + (mods.bits() as u16),
            Self::TapDance(n) => ranges::QK_TAP_DANCE.start + (*n as u16),
            Self::Macro(n) => ranges::QK_MACRO.start + (*n as u16),
            Self::CustomKb(n) => ranges::QK_KB.start + (*n as u16),
            Self::CustomUser(n) => ranges::QK_USER.start + *n,
            Self::Raw(code) => *code,
        }
    }

    /// Returns the target layer index if this keycode is a layer-targeting action.
    pub fn layer(&self) -> Option<u8> {
        match self {
            Self::LayerTap { layer, .. }
            | Self::LayerMod { layer, .. }
            | Self::LayerOp { layer, .. } => Some(*layer),
            _ => None,
        }
    }

    /// Returns the modifier mask if this keycode incorporates modifiers.
    pub fn mods(&self) -> Option<QmkModMask> {
        match self {
            Self::ModCombo { mods, .. }
            | Self::ModTap { mods, .. }
            | Self::LayerMod { mods, .. }
            | Self::OneShotMod(mods) => Some(*mods),
            _ => None,
        }
    }

    /// Returns the 8-bit base keycode if this keycode contains one.
    pub fn base_keycode(&self) -> Option<u8> {
        match self {
            Self::ModCombo { keycode, .. }
            | Self::ModTap { keycode, .. }
            | Self::LayerTap { keycode, .. } => Some(*keycode),
            Self::Keycode(kc) => {
                let val = *kc as u16;
                if val <= 0xFF {
                    Some(val as u8)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Returns the category of this keycode, if applicable.
    pub fn category(&self) -> Option<KeycodeCategory> {
        match self {
            Self::Keycode(kc) => Some(kc.category()),
            Self::ModCombo { .. } | Self::ModTap { .. } => Some(KeycodeCategory::Basic),
            Self::TapDance(_) => Some(KeycodeCategory::Special),
            Self::Macro(_) | Self::CustomKb(_) | Self::CustomUser(_) => {
                Some(KeycodeCategory::Custom)
            }
            _ => None,
        }
    }
}

impl From<u16> for QmkKeycode {
    fn from(code: u16) -> Self {
        Self::from_u16(code)
    }
}

impl From<QmkKeycode> for u16 {
    fn from(kc: QmkKeycode) -> Self {
        kc.to_u16()
    }
}

impl From<Keycode> for QmkKeycode {
    fn from(kc: Keycode) -> Self {
        Self::Keycode(kc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_encoding_and_roundtrip() {
        for op in QmkLayerOp::ALL {
            for layer in 0..16 {
                let code = op.encode(layer).unwrap();
                let decoded = QmkKeycode::from_u16(code);
                assert_eq!(decoded, QmkKeycode::LayerOp { op, layer });
                assert_eq!(decoded.to_u16(), code);
            }
        }
    }

    #[test]
    fn test_mod_combo_roundtrip() {
        let mods = QmkModMask::from_bits(QmkModMask::LSFT);
        let code = encode_mod_combo(mods, 0x2A).unwrap();
        let decoded = QmkKeycode::from_u16(code);
        assert_eq!(
            decoded,
            QmkKeycode::ModCombo {
                mods,
                keycode: 0x2A
            }
        );
        assert_eq!(decoded.to_u16(), code);
    }

    #[test]
    fn test_mod_tap_roundtrip() {
        let mods = QmkModMask::from_bits(QmkModMask::LSFT | QmkModMask::LALT);
        let code = encode_mod_tap(mods, 0x04).unwrap();
        let decoded = QmkKeycode::from_u16(code);
        assert_eq!(
            decoded,
            QmkKeycode::ModTap {
                mods,
                keycode: 0x04
            }
        );
        assert_eq!(decoded.to_u16(), code);
    }

    #[test]
    fn test_layer_tap_roundtrip() {
        let code = encode_layer_tap(3, 0x1C).unwrap();
        let decoded = QmkKeycode::from_u16(code);
        assert_eq!(
            decoded,
            QmkKeycode::LayerTap {
                layer: 3,
                keycode: 0x1C
            }
        );
        assert_eq!(decoded.to_u16(), code);
    }

    #[test]
    fn test_layer_mod_roundtrip() {
        let mods = QmkModMask::from_bits(QmkModMask::LSFT | QmkModMask::LCTL);
        let code = encode_layer_mod(2, mods).unwrap();
        let decoded = QmkKeycode::from_u16(code);
        assert_eq!(decoded, QmkKeycode::LayerMod { layer: 2, mods });
        assert_eq!(decoded.to_u16(), code);
    }

    #[test]
    fn test_one_shot_mod_roundtrip() {
        let mods = QmkModMask::from_bits(QmkModMask::LCTL | QmkModMask::RIGHT_HAND);
        let code = encode_one_shot_mod(mods).unwrap();
        let decoded = QmkKeycode::from_u16(code);
        assert_eq!(decoded, QmkKeycode::OneShotMod(mods));
        assert_eq!(decoded.to_u16(), code);
    }

    #[test]
    fn test_basic_and_quantum_keycodes() {
        assert_eq!(
            QmkKeycode::from_u16(Keycode::KC_A as u16),
            QmkKeycode::Keycode(Keycode::KC_A)
        );
        assert_eq!(
            QmkKeycode::from_u16(Keycode::QK_BOOTLOADER as u16),
            QmkKeycode::Keycode(Keycode::QK_BOOTLOADER)
        );
    }

    #[test]
    fn test_helper_accessors() {
        let mt = QmkKeycode::from_u16(0x2000 | (0x02 << 8) | 0x04);
        assert_eq!(mt.base_keycode(), Some(0x04));
        assert_eq!(mt.mods(), Some(QmkModMask::from_bits(0x02)));
        assert_eq!(mt.layer(), None);

        let lt = QmkKeycode::from_u16(0x4000 | (3 << 8) | 0x28);
        assert_eq!(lt.base_keycode(), Some(0x28));
        assert_eq!(lt.layer(), Some(3));
        assert_eq!(lt.mods(), None);

        let ralt_mods = QmkModMask::from_bits(QmkModMask::RALT);
        assert!(ralt_mods.has_alt());
        assert!(ralt_mods.is_right());
    }
}
