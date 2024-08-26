use bevy::prelude::*;

pub enum DescribableType {
    Planet,
    Star,
}

use super::{Empire,Colony};

pub fn update_descriptions_system (
    mut query : Query<(&mut Description,&Colony), Changed<Colony>>,
    empire_query : Query<&Empire,Without<Colony>>
) {
    for (mut description,colony) in query.iter_mut() {
        if let Ok(owner) = empire_query.get(colony.owner) {
            description.empire_color = Some(owner.color);
        }

        // todo.. account for colony abandonment
    }
}

#[derive(Component)]
pub struct Description {
    pub name : String,
    pub describable_type : DescribableType,
    pub empire_color : Option<Color>,
}

impl Description {
    pub fn planet(name : String) -> Self {
        Self {
            name,
            describable_type : DescribableType::Planet,
            empire_color : None
        }
    }
    pub fn star(name : String) -> Self {
        Self {
            name,
            describable_type : DescribableType::Star,
            empire_color : None
        }
    }

    pub fn type_name(&self) -> &str {
        match self.describable_type {
            DescribableType::Planet => "Planet",
            DescribableType::Star => "Star"
        }
    }
    pub fn type_color(&self) -> Color {
        match self.describable_type {
            DescribableType::Planet => Color::srgb(0.,1.,0.),
            DescribableType::Star => Color::srgb(1.,165./255.,0.)
        }
    }

    // Generating the description typically requires retrieving additional information (the Star struct for a Star etc.)
    // It's awkward for Description itself to be querying that information. (more from code-writing than performance standpoint)
    //
    // Most ergonomic way is for get_description to be a member of the specific special struct (star, planet, etc.)
}