use dioxus::prelude::*;
use crate::Route;

#[component]
pub fn HomePage() -> Element {
    rsx! {
        div {
            class: "container",
            style: "padding: 20px; font-family: system-ui, -apple-system, sans-serif;",
            
            header {
                style: "text-align: center; margin-bottom: 40px;",
                h1 { 
                    style: "font-size: 2.5em; margin-bottom: 10px;",
                    "💪 LogOut" 
                }
                p { 
                    style: "font-size: 1.2em; color: #666;",
                    "Turn off your computer, Log your workOut" 
                }
            }
            
            nav {
                style: "display: flex; flex-direction: column; gap: 15px; max-width: 400px; margin: 0 auto;",
                
                Link {
                    to: Route::WorkoutLogPage {},
                    style: "
                        display: block;
                        padding: 20px;
                        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                        color: white;
                        text-decoration: none;
                        border-radius: 10px;
                        text-align: center;
                        font-size: 1.2em;
                        font-weight: bold;
                        box-shadow: 0 4px 6px rgba(0,0,0,0.1);
                    ",
                    "🏋️ Start New Workout"
                }
                
                Link {
                    to: Route::ExerciseListPage {},
                    style: "
                        display: block;
                        padding: 20px;
                        background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
                        color: white;
                        text-decoration: none;
                        border-radius: 10px;
                        text-align: center;
                        font-size: 1.2em;
                        font-weight: bold;
                        box-shadow: 0 4px 6px rgba(0,0,0,0.1);
                    ",
                    "📚 Browse Exercises"
                }
            }
            
            footer {
                style: "text-align: center; margin-top: 60px; color: #999; font-size: 0.9em;",
                p { 
                    "Exercise database from "
                    a { 
                        href: "https://github.com/yuhonas/free-exercise-db",
                        target: "_blank",
                        style: "color: #667eea;",
                        "free-exercise-db" 
                    }
                }
            }
        }
    }
}
