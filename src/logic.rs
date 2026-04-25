#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DozenalDigit {
    D0, D1, D2, D3, D4, D5, D6, D7, D8, D9, D10, D11
}

impl DozenalDigit {
    // Wandelt eine Dozenal-Ziffer in ihren dezimalen Wert um
    pub fn to_value(self) -> u32 {
        match self {
            DozenalDigit::D0 => 0,
            DozenalDigit::D1 => 1,  // Ankerpunkt: Pfeil hoch ^
            DozenalDigit::D2 => 2,
            DozenalDigit::D3 => 3,
            DozenalDigit::D4 => 4,  // Ankerpunkt: Pfeil links <
            DozenalDigit::D5 => 5,
            DozenalDigit::D6 => 6,
            DozenalDigit::D7 => 7,  // Ankerpunkt: Pfeil rechts >
            DozenalDigit::D8 => 8,
            DozenalDigit::D9 => 9,
            DozenalDigit::D10 => 10, // Ankerpunkt: Pfeil runter v
            DozenalDigit::D11 => 11,
        }
    }

    // Erstellt eine Ziffer aus einem Wert (0-11)
    pub fn from_value(val: u32) -> Option<Self> {
        match val {
            0 => Some(DozenalDigit::D0),
            1 => Some(DozenalDigit::D1),
            2 => Some(DozenalDigit::D2),
            3 => Some(DozenalDigit::D3),
            4 => Some(DozenalDigit::D4),
            5 => Some(DozenalDigit::D5),
            6 => Some(DozenalDigit::D6),
            7 => Some(DozenalDigit::D7),
            8 => Some(DozenalDigit::D8),
            9 => Some(DozenalDigit::D9),
            10 => Some(DozenalDigit::D10),
            11 => Some(DozenalDigit::D11),
            _ => None,
        }
    }
}

// Die Konvertierungs-Einheit
pub struct DozenalConverter;

impl DozenalConverter {
    // Macht aus einer Liste von Ziffern eine Dezimalzahl
    // Beispiel: [D1, D0] -> 1 * 12^1 + 0 * 12^0 = 12
    pub fn to_decimal(digits: &[DozenalDigit]) -> f64 {
        let mut result = 0.0;
        for (i, digit) in digits.iter().rev().enumerate() {
            result += digit.to_value() as f64 * 12.0_f64.powi(i as i32);
        }
        result
    }

    // Macht aus einer Dezimalzahl eine Liste von Dozenal-Ziffern
    // Beispiel: 14 -> 14 / 12 = 1 Rest 2 -> [D1, D2]
    pub fn from_decimal(value: f64) -> Vec<DozenalDigit> {
        if value == 0.0 { return vec![DozenalDigit::D0]; }
        
        let mut digits = Vec::new();
        let mut integer_part = value.floor() as u64;
        
        while integer_part > 0 {
            let remainder = (integer_part % 12) as u32;
            if let Some(d) = DozenalDigit::from_value(remainder) {
                digits.push(d);
            }
            integer_part /= 12;
        }
        digits.reverse();
        digits
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion() {
        // Test: 14 dezimal sollte Dozenal 12 sein (Pfeil hoch ^ + 2 Halbkreise)
        let dec = 14.0;
        let doz = DozenalConverter::from_decimal(dec);
        assert_eq!(doz, vec![DozenalDigit::D1, DozenalDigit::D2]);

        // Test zurück: Dozenal [D1, D4] (14 dozenal) sollte 16 dezimal sein
        let doz_input = vec![DozenalDigit::D1, DozenalDigit::D4];
        let dec_result = DozenalConverter::to_decimal(&doz_input);
        assert_eq!(dec_result, 16.0);
    }
}
