What's new in 0.8.1
- **Updates tells you about all three pieces.** The pane now has a row each for this app, pi and Gentle Shell, with the installed version, the newest one, and its own button. Gentle Shell is compared against npm's stable release; a build that is not a plain stable version (a main-channel build, say) is never reported as behind.
- **Update Gentle Shell.** When the shell is behind, one button opens the terminal on `gentle-shell upgrade`, so you read what it does, like the existing Update.
- **Skip this version.** A version you skip is not offered again when the app opens; a newer one is. Settings still shows it, with a way to stop skipping.
- **It says when it last checked, and why it could not.** "Last checked: 3m ago", and when a lookup fails the row says which one and why ("offline", "the registry did not answer") instead of going blank. An unknown is never shown as "up to date".
- **Update extensions is honest.** It stays one button in its own block, now saying that this Gentle Shell cannot tell which extensions are behind.
- **Opening Settings no longer flashes terminal windows.** The reads it runs in the background (versions, sign-in checks, usage) now run without a console. The terminals of the Update actions stay visible on purpose.
- **Account stops re-reading in a loop.** Returning to the window asks for one read per ten seconds instead of one per focus, which also ends the loop of terminals when minimising and restoring.

Novedades en 0.8.1
- **Actualizaciones te avisa de las tres piezas.** El panel ahora tiene una fila para esta app, otra para pi y otra para Gentle Shell, con la versión instalada, la más nueva y su propio botón. Gentle Shell se compara con la versión estable de npm; una compilación que no es una versión estable simple (una del canal main, por ejemplo) nunca se informa como atrasada.
- **Actualizar Gentle Shell.** Cuando el shell está atrasado, un botón abre la terminal con `gentle-shell upgrade`, así lees lo que hace, igual que el Actualizar de siempre.
- **Omitir esta versión.** Una versión que omitís no se vuelve a ofrecer al abrir la app; una más nueva sí. Configuración la sigue mostrando, con una forma de dejar de omitirla.
- **Dice cuándo comprobó por última vez, y por qué no pudo.** "Última comprobación: hace 3m", y cuando una consulta falla la fila dice cuál y por qué ("sin conexión", "el registro no respondió") en vez de quedar en blanco. Un dato desconocido nunca se muestra como "al día".
- **Actualizar extensiones es honesto.** Sigue siendo un botón, en su propio bloque, y ahora dice que este Gentle Shell no puede saber cuáles extensiones están atrasadas.
- **Abrir Configuración ya no parpadea terminales.** Las lecturas que hace en segundo plano (versiones, revisión de cuentas, uso) corren sin consola. Las terminales de las acciones de Actualizar siguen visibles a propósito.
- **Cuenta deja de releer en bucle.** Volver a la ventana pide una lectura cada diez segundos en vez de una por cada foco, lo que también termina con el bucle de terminales al minimizar y restaurar.
