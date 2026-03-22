use serde::{Deserialize, Serialize};
/// Exercise category (e.g. strength, cardio).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    strum::EnumIter,
    strum::Display,
    strum::AsRefStr,
)]
#[strum(serialize_all = "lowercase")]
pub enum Category {
    #[serde(rename = "cardio")]
    Cardio,
    #[serde(rename = "olympic weightlifting")]
    #[strum(to_string = "olympic weightlifting")]
    OlympicWeightlifting,
    #[serde(rename = "plyometrics")]
    Plyometrics,
    #[serde(rename = "powerlifting")]
    Powerlifting,
    #[serde(rename = "strength")]
    Strength,
    #[serde(rename = "stretching")]
    Stretching,
    #[serde(rename = "strongman")]
    Strongman,
}
/// The primary muscular force direction of an exercise.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    strum::EnumIter,
    strum::Display,
    strum::AsRefStr,
)]
#[strum(serialize_all = "lowercase")]
pub enum Force {
    #[serde(rename = "pull")]
    Pull,
    #[serde(rename = "push")]
    Push,
    #[serde(rename = "static")]
    Static,
}
impl Force {
    /// Returns true if reps are applicable for this force type.
    pub fn has_reps(self) -> bool {
        matches!(self, Self::Pull | Self::Push)
    }
}
/// The difficulty level of an exercise.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    strum::EnumIter,
    strum::Display,
    strum::AsRefStr,
)]
#[strum(serialize_all = "lowercase")]
pub enum Level {
    #[serde(rename = "beginner")]
    Beginner,
    #[serde(rename = "intermediate")]
    Intermediate,
    #[serde(rename = "expert")]
    Expert,
}
/// Whether an exercise is compound (multi-joint) or isolation (single-joint).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    strum::EnumIter,
    strum::Display,
    strum::AsRefStr,
)]
#[strum(serialize_all = "lowercase")]
pub enum Mechanic {
    #[serde(rename = "compound")]
    Compound,
    #[serde(rename = "isolation")]
    Isolation,
}
/// The primary equipment required to perform an exercise.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    strum::EnumIter,
    strum::Display,
    strum::AsRefStr,
)]
#[strum(serialize_all = "lowercase")]
pub enum Equipment {
    #[serde(rename = "bands")]
    Bands,
    #[serde(rename = "barbell")]
    Barbell,
    #[serde(rename = "body only")]
    #[strum(to_string = "body only")]
    BodyOnly,
    #[serde(rename = "cable")]
    Cable,
    #[serde(rename = "dumbbell")]
    Dumbbell,
    #[serde(rename = "e-z curl bar")]
    #[strum(to_string = "e-z curl bar")]
    EzCurlBar,
    #[serde(rename = "exercise ball")]
    #[strum(to_string = "exercise ball")]
    ExerciseBall,
    #[serde(rename = "foam roll")]
    #[strum(to_string = "foam roll")]
    FoamRoll,
    #[serde(rename = "kettlebells")]
    Kettlebells,
    #[serde(rename = "machine")]
    Machine,
    #[serde(rename = "medicine ball")]
    #[strum(to_string = "medicine ball")]
    MedicineBall,
    #[serde(rename = "other")]
    Other,
}
/// A muscle or muscle group targeted by an exercise.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    strum::EnumIter,
    strum::Display,
    strum::AsRefStr,
)]
#[strum(serialize_all = "lowercase")]
pub enum Muscle {
    #[serde(rename = "abdominals")]
    Abdominals,
    #[serde(rename = "abductors")]
    Abductors,
    #[serde(rename = "adductors")]
    Adductors,
    #[serde(rename = "biceps")]
    Biceps,
    #[serde(rename = "calves")]
    Calves,
    #[serde(rename = "chest")]
    Chest,
    #[serde(rename = "forearms")]
    Forearms,
    #[serde(rename = "glutes")]
    Glutes,
    #[serde(rename = "hamstrings")]
    Hamstrings,
    #[serde(rename = "lats")]
    Lats,
    #[serde(rename = "lower back")]
    #[strum(to_string = "lower back")]
    LowerBack,
    #[serde(rename = "middle back")]
    #[strum(to_string = "middle back")]
    MiddleBack,
    #[serde(rename = "neck")]
    Neck,
    #[serde(rename = "quadriceps")]
    Quadriceps,
    #[serde(rename = "shoulders")]
    Shoulders,
    #[serde(rename = "traps")]
    Traps,
    #[serde(rename = "triceps")]
    Triceps,
}
#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;
    #[test]
    fn category_round_trip() {
        let json = serde_json::to_string(&Category::OlympicWeightlifting).unwrap();
        assert_eq!(json, "\"olympic weightlifting\"");
        let back: Category = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Category::OlympicWeightlifting);
    }
    #[test]
    fn equipment_round_trip() {
        let json = serde_json::to_string(&Equipment::BodyOnly).unwrap();
        assert_eq!(json, "\"body only\"");
        let back: Equipment = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Equipment::BodyOnly);
    }
    #[test]
    fn muscle_round_trip() {
        let json = serde_json::to_string(&Muscle::LowerBack).unwrap();
        assert_eq!(json, "\"lower back\"");
        let back: Muscle = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Muscle::LowerBack);
    }
    #[test]
    fn force_has_reps() {
        assert!(Force::Push.has_reps());
        assert!(Force::Pull.has_reps());
        assert!(!Force::Static.has_reps());
    }
    #[test]
    fn category_display_all_variants() {
        assert_eq!(Category::Cardio.to_string(), "cardio");
        assert_eq!(
            Category::OlympicWeightlifting.to_string(),
            "olympic weightlifting"
        );
        assert_eq!(Category::Plyometrics.to_string(), "plyometrics");
        assert_eq!(Category::Powerlifting.to_string(), "powerlifting");
        assert_eq!(Category::Strength.to_string(), "strength");
        assert_eq!(Category::Stretching.to_string(), "stretching");
        assert_eq!(Category::Strongman.to_string(), "strongman");
    }
    #[test]
    fn force_display_all_variants() {
        assert_eq!(Force::Pull.to_string(), "pull");
        assert_eq!(Force::Push.to_string(), "push");
        assert_eq!(Force::Static.to_string(), "static");
    }
    #[test]
    fn level_display_all_variants() {
        assert_eq!(Level::Beginner.to_string(), "beginner");
        assert_eq!(Level::Intermediate.to_string(), "intermediate");
        assert_eq!(Level::Expert.to_string(), "expert");
    }
    #[test]
    fn mechanic_display_all_variants() {
        assert_eq!(Mechanic::Compound.to_string(), "compound");
        assert_eq!(Mechanic::Isolation.to_string(), "isolation");
    }
    #[test]
    fn equipment_display_all_variants() {
        assert_eq!(Equipment::Bands.to_string(), "bands");
        assert_eq!(Equipment::Barbell.to_string(), "barbell");
        assert_eq!(Equipment::BodyOnly.to_string(), "body only");
        assert_eq!(Equipment::Cable.to_string(), "cable");
        assert_eq!(Equipment::Dumbbell.to_string(), "dumbbell");
        assert_eq!(Equipment::EzCurlBar.to_string(), "e-z curl bar");
        assert_eq!(Equipment::ExerciseBall.to_string(), "exercise ball");
        assert_eq!(Equipment::FoamRoll.to_string(), "foam roll");
        assert_eq!(Equipment::Kettlebells.to_string(), "kettlebells");
        assert_eq!(Equipment::Machine.to_string(), "machine");
        assert_eq!(Equipment::MedicineBall.to_string(), "medicine ball");
        assert_eq!(Equipment::Other.to_string(), "other");
    }
    #[test]
    fn muscle_display_all_variants() {
        assert_eq!(Muscle::Abdominals.to_string(), "abdominals");
        assert_eq!(Muscle::Abductors.to_string(), "abductors");
        assert_eq!(Muscle::Adductors.to_string(), "adductors");
        assert_eq!(Muscle::Biceps.to_string(), "biceps");
        assert_eq!(Muscle::Calves.to_string(), "calves");
        assert_eq!(Muscle::Chest.to_string(), "chest");
        assert_eq!(Muscle::Forearms.to_string(), "forearms");
        assert_eq!(Muscle::Glutes.to_string(), "glutes");
        assert_eq!(Muscle::Hamstrings.to_string(), "hamstrings");
        assert_eq!(Muscle::Lats.to_string(), "lats");
        assert_eq!(Muscle::LowerBack.to_string(), "lower back");
        assert_eq!(Muscle::MiddleBack.to_string(), "middle back");
        assert_eq!(Muscle::Neck.to_string(), "neck");
        assert_eq!(Muscle::Quadriceps.to_string(), "quadriceps");
        assert_eq!(Muscle::Shoulders.to_string(), "shoulders");
        assert_eq!(Muscle::Traps.to_string(), "traps");
        assert_eq!(Muscle::Triceps.to_string(), "triceps");
    }
    #[test]
    fn category_all_contains_every_variant() {
        assert_eq!(Category::iter().count(), 7);
    }
    #[test]
    fn force_all_contains_every_variant() {
        assert_eq!(Force::iter().count(), 3);
    }
    #[test]
    fn equipment_all_contains_every_variant() {
        assert_eq!(Equipment::iter().count(), 12);
    }
    #[test]
    fn muscle_all_contains_every_variant() {
        assert_eq!(Muscle::iter().count(), 17);
    }
    #[test]
    fn all_categories_serde_round_trip() {
        for cat in Category::iter() {
            let json = serde_json::to_string(&cat).unwrap();
            let back: Category = serde_json::from_str(&json).unwrap();
            assert_eq!(back, cat);
        }
    }
    #[test]
    fn all_forces_serde_round_trip() {
        for f in Force::iter() {
            let json = serde_json::to_string(&f).unwrap();
            let back: Force = serde_json::from_str(&json).unwrap();
            assert_eq!(back, f);
        }
    }
    #[test]
    fn all_equipment_serde_round_trip() {
        for eq in Equipment::iter() {
            let json = serde_json::to_string(&eq).unwrap();
            let back: Equipment = serde_json::from_str(&json).unwrap();
            assert_eq!(back, eq);
        }
    }
    #[test]
    fn all_muscles_serde_round_trip() {
        for m in Muscle::iter() {
            let json = serde_json::to_string(&m).unwrap();
            let back: Muscle = serde_json::from_str(&json).unwrap();
            assert_eq!(back, m);
        }
    }
    #[test]
    fn level_serde_round_trip() {
        for level in [Level::Beginner, Level::Intermediate, Level::Expert] {
            let json = serde_json::to_string(&level).unwrap();
            let back: Level = serde_json::from_str(&json).unwrap();
            assert_eq!(back, level);
        }
    }
    #[test]
    fn mechanic_serde_round_trip() {
        for mech in [Mechanic::Compound, Mechanic::Isolation] {
            let json = serde_json::to_string(&mech).unwrap();
            let back: Mechanic = serde_json::from_str(&json).unwrap();
            assert_eq!(back, mech);
        }
    }
    #[test]
    fn level_all_contains_every_variant() {
        assert_eq!(Level::iter().count(), 3);
        assert!(Level::iter().any(|l| l == Level::Beginner));
        assert!(Level::iter().any(|l| l == Level::Intermediate));
        assert!(Level::iter().any(|l| l == Level::Expert));
    }
    #[test]
    fn mechanic_all_contains_every_variant() {
        assert_eq!(Mechanic::iter().count(), 2);
        assert!(Mechanic::iter().any(|m| m == Mechanic::Compound));
        assert!(Mechanic::iter().any(|m| m == Mechanic::Isolation));
    }
}
