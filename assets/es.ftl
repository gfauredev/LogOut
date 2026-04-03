# LogOut – Traducciones al español

## Para toda la aplicación
app-title = 💪 LogOut
app-subtitle = Apaga tu ordenador, registra tu entrenamiento

## Página de inicio
no-sessions = Sin sesiones pasadas
start-first-workout = Pulsa + para empezar tu primer entrenamiento
start-new-workout = Nuevo entrenamiento
session-repeat-title = Iniciar nueva sesión basada en esta
session-repeat-weekday-title = Repetir la sesión del mismo día de la semana
session-delete-title = Eliminar sesión
session-show-more = +{ $count } más
session-delete-confirm = ¿Eliminar esta sesión?
session-delete-confirm-btn = 🗑️ Eliminar
cancel-btn = ❌ Cancelar

## Página de ejercicios
browse-exercises = { $count } ejercicios disponibles
search-placeholder = Buscar ejercicios por nombre o atributos
add-exercise = Añadir ejercicio personalizado
filter-add = Activar filtro
filter-remove = Eliminar filtro

## Tarjeta de ejercicio
exercise-edit = Editar
exercise-clone = Duplicar y editar

## Sesión activa – búsqueda
session-search-placeholder = Buscar un ejercicio...
session-add-exercise-title = Añadir ejercicio personalizado
session-filter-remove = Eliminar filtro
session-filter-add = Añadir filtro
pending-more = Más preañadidos ({ $count })

## Sesión activa – encabezado
session-title = ⏱️ Sesión activa
session-timer-title = Clic para configurar la duración del descanso
session-pause-btn = Pausar sesión
session-resume-btn = Reanudar sesión
session-cancel-btn = Cancelar sesión
session-finish-btn = Finalizar sesión

## Sesión activa – duración del descanso
rest-duration-aria = Configurar duración del descanso
rest-duration-label = Duración del descanso

## Sesión activa – ejercicios completados
completed-exercises-title = Ejercicios completados

## Formulario de ejercicio
exercise-complete-title = Completar ejercicio
time-placeholder = mm:ss
weight-placeholder = kg
distance-placeholder = km
reps-placeholder = rep.

## Registro de ejercicio completado
log-replay-title = Hacer otra serie
log-edit-title = Editar este ejercicio
log-delete-title = Eliminar este ejercicio

## Páginas de añadir / editar ejercicio
add-exercise-page-title = Añadir ejercicio
edit-exercise-page-title = Editar ejercicio
exercise-not-found = Ejercicio no encontrado
exercise-save = Guardar ejercicio
exercise-save-changes = Guardar cambios
cancel-title = Cancelar

## Campos del formulario de ejercicio
form-name-label = Nombre del ejercicio *
form-category-label = Categoría *
form-force-label = Tipo de fuerza
form-equipment-label = Equipamiento
form-muscles-primary-label = Músculos principales
form-muscles-secondary-label = Músculos secundarios
form-instructions-label = Instrucciones
form-images-label = Imágenes
form-name-placeholder = Flexiones
form-muscle-select-default = Seleccionar músculo...
form-instruction-placeholder = Añadir un paso de instrucción...
form-image-url-placeholder = https://example.com/imagen.jpg
form-none-option = Ninguno
form-image-upload-title = Subir una imagen local
form-local-image-placeholder = /ruta/a/imagen.jpg
form-local-image-title = Ruta a un archivo de imagen local (se copiará al almacenamiento de la aplicación)
form-save-aria = Guardar

## Página Más
more-title = ⚙️ Más
more-export-section = 📤 Exportar
more-export-exercises-btn = 💾 { $count } Ejercicios personalizados
more-export-sessions-btn = 💾 { $count } Sesiones
more-import-section = 📥 Importar
more-import-exercises-btn = 📂 Ejercicios personalizados
more-import-sessions-btn = 📂 Sesiones
more-about-section = LogOut
more-about-desc-a = Una aplicación simple, eficiente y multiplataforma para registrar entrenamientos con
more-about-exercises-link = 800+ ejercicios
more-about-desc-b = integrados, por
more-db-url-section = ⚙️ URL de la base de datos de ejercicios
more-db-url-desc = Cambia la fuente de datos de ejercicios. Guarda para descargar desde esta URL.
more-db-url-save-aria = Guardar
more-db-exercises-count = 📦 { $count } ejercicios
more-db-images-count = 🖼️ { $count } imágenes
more-oss-section = Código abierto y licencias
more-oss-desc-a = Este proyecto es de código abierto bajo la licencia GPL-3.0 y utiliza otros proyectos de código abierto. Consulta su
more-oss-repo-link = repositorio de código
more-oss-desc-b = para más detalles. Aceptamos contribuciones, incluyendo a la
more-oss-db-link = base de datos de ejercicios
more-built-with-section = Creado con
more-built-with-rust = Lenguaje de programación de sistemas
more-built-with-dioxus = Framework Rust para aplicaciones multiplataforma
more-built-with-freeexdb = Datos e imágenes de ejercicios, por yuhonas
more-built-with-others = Y muchos más …
more-replace-confirm = ¿Reemplazar el ejercicio personalizado { $name }?
more-replace-btn = 💾 Reemplazar
more-sessions-refused = sesión/sesiones rechazada(s): ID ya existe
more-exercises-refused = ejercicio(s) rechazado(s): conflicto con ID integrado

## Mensajes toast (prefijo estático; el detalle técnico se añade en tiempo de ejecución)
toast-export-failed = ⚠️ Error al exportar
toast-export-sessions-failed = ⚠️ Error al exportar sesiones
toast-sessions-invalid = ⚠️ JSON de sesiones no válido
toast-exercises-invalid = ⚠️ JSON de ejercicios no válido
db-empty-toast = 📥 Base de datos de ejercicios vacía — toca para descargar

## Etiquetas de fecha relativa
date-today = Hoy
date-yesterday = Ayer
date-days-ago = Hace { $count } día{ $count ->
    [one] {""}
   *[other] s
}

## Toast de felicitación y notificaciones
congratulations = 🎉 ¡Buen entrenamiento! ¡Sesión completada!
notif-permission-blocked = ⚠️ Notificaciones bloqueadas
notif-permission-enable = ⚠️ Pulsa aquí para activar las notificaciones
notif-duration-title = Duración alcanzada
notif-duration-body = ¡Duración objetivo del ejercicio alcanzada!
notif-rest-title = Descanso terminado
notif-rest-body = ¡Es hora de tu próxima serie!

## Página de estadísticas
analytics-title = 📊 Estadísticas
analytics-subtitle = Sigue tu progreso a lo largo del tiempo
analytics-pairs-label = Pares métrica–ejercicio (⩽ 8)
analytics-empty = Selecciona ejercicios para ver las estadísticas
analytics-metric-weight = Peso (kg)
analytics-metric-reps = Repeticiones
analytics-metric-distance = Distancia
analytics-metric-duration = Duración
analytics-select-exercise = -- Seleccionar ejercicio --
analytics-remove-series = Eliminar esta serie
