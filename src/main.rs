use gtk::prelude::*;
use gtk::{Menu, MenuItem};
use libappindicator::{AppIndicator, AppIndicatorStatus};
use std::fs;
use std::time::Duration;
use std::error::Error;
use glib::{timeout_add_local, Continue};

fn get_cpu_temperature() -> Result<f32, Box<dyn Error>> {
    let possible_paths = [
        "/sys/class/thermal/thermal_zone0/temp",
        "/sys/class/hwmon/hwmon0/temp1_input",
        "/sys/class/hwmon/hwmon1/temp1_input",
    ];

    for path in possible_paths.iter() {
        if let Ok(content) = fs::read_to_string(path) {
            let temp_millicelsius = content.trim().parse::<f32>()?;
            return Ok(temp_millicelsius / 1000.0);
        }
    }

    Err("No se pudo encontrar el archivo de temperatura del CPU".into())
}

fn main() {
    if gtk::init().is_err() {
        eprintln!("Error al inicializar GTK");
        return;
    }

    // Crear el menú contextual como mutable
    let mut menu = Menu::new();
    
    // Añadir item "Salir" al menú
    let quit_item = MenuItem::with_label("Exit");
    menu.append(&quit_item);
    
    // Mostrar todos los items del menú
    menu.show_all();

    // Configurar el indicador del system tray
    let mut indicator = AppIndicator::new("cpu-temperature", "weather-clear");
    indicator.set_status(AppIndicatorStatus::Active);
    indicator.set_menu(&mut menu);

    // Configurar la acción para cerrar la aplicación
    quit_item.connect_activate(|_| {
        gtk::main_quit();
    });

    // Actualizar temperatura periódicamente
    {
        let mut indicator = indicator;
        timeout_add_local(Duration::from_secs(2), move || {
            match get_cpu_temperature() {
                Ok(temp) => {
                    // Mostrar temperatura como entero (sin decimales)
                    let label = format!("{}°C", temp.round() as i32);
                    indicator.set_label(&label, "");
                    
                    let (icon_name, tooltip) = if temp > 80.0 {
                        ("weather-storm", "¡Temperatura crítica!")
                    } else if temp > 60.0 {
                        ("weather-overcast", "Temperatura alta")
                    } else {
                        ("weather-clear", "Temperatura normal")
                    };
                    
                    indicator.set_icon(icon_name);
                    indicator.set_title(tooltip);
                },
                Err(e) => eprintln!("Error al leer temperatura: {}", e),
            }
            Continue(true)
        });
    }

    // Ejecutar el main loop de GTK
    gtk::main();
}