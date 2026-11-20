// The rose the welcome screen blooms: braille art plus a 0-9 shade map, taken
// as-is from gentle-shell-gpui - `crates/app/src/shared/banner.rs` draws these
// same two arrays as ROSE and ROSE_SHADE - whose workspace declares
// `license = "MIT"`, the same licence as this app:
// github.com/Gentleman-Programming/gentle-shell-gpui.
//
// A character of `ROSE` is a braille cell: 2 columns by 4 rows of dots, so one
// row of the art is four rows of dots. `SHADE` puts each cell on a 0-9
// brightness scale, read by column; a cell with no digit is a mid shade. The
// dots are decoded, and their bloom is timed, in ./rose.ts.

export const ROSE: string[] = ["            ⣠⣿⣿⣶⣶⣤⣤⣤⣤⣤⣤⣤⣀    ⢀⡀","         ⢀⣴⣾⣿⣿⠟⠝⢙⣙⣙⣙⣙⢙⣿⣷⢿⢿⢿⢿⢿⣿⣷⣄⡀","      ⣀⣠⣴⣿⣿⣿⠟⢡⣶⡿⠟⢟⡽⣿⣿⣿⣿⡟⢶⣤⣤⣄⣁⠁⠙⢿⣿⣦","   ⢀⣴⣿⣿⠅⣿⣿⣿⠅⢀⣿⠏ ⢰⡏⢹⢯⡤⢖⣯⣍⠻⡍⢻⡌⢽⣷⠄ ⢿⣿⡷⢶⣤⣤","⢀⣀⣴⣿⣿⣿⠟ ⠹⣿⣿  ⢻⣇ ⠘⠻⣦⣈⢙⣻⣿⣷⣾⡥⠜⢁⣼⠟⠁⢴⣿⡿⠁⢽⣿⣿⣄","⢻⣿⢿⣿⣿⣟⠅  ⠘⢿⣧ ⠘⠙⢦⣀  ⠙⠳⢤  ⢀⡴⠞⠛⠝⢀⣴⣿⠟  ⢽⣿⣿⢿⡆"," ⣿⢝⢿⣿⣧⠁   ⢸⠍⠳⣄  ⠉⠓⠦⢄⢀⣀⣁⣤⣥⣤⣤⣶⠿⢿⡟⠁   ⣽⣿⢿⣽⡇"," ⢿⣟⠝⢿⣿⣷⣄  ⠝⠕⠅⠈⠂ ⢀⡤⠖⠛⠉⠉⡁⣀   ⢀⠅⢽⠇  ⢀⣽⣿⣿⣿⠏"," ⠈⠻⢷⣝⣝⣿⢿⢷⣤⣄    ⠴⠉⠄⢔⢴⢾⢿⢿⢿⢗⠔⠔⠄⠁ ⢁⢄⣵⣿⡿⠟⢉⢽⣷⡄","    ⠈⠉⠉⢿⡛⠿⢿⣿⣶⣤⣀⡁  ⠁⠁⠙⠅⠙⠁⠁⠁ ⣀⣴⣾⢿⠿⠛⠉ ⢤⣽⣿⣿⣇","       ⠈⢿⣧⡄ ⠁⠉⠛⠿⠿⢷⣶⣤⣀   ⢀⡴⠛⠉⠁ ⠁ ⢀⣴⢿⢿⣽⠟⠉⠁","         ⠙⠻⣷⣥⣄⢀    ⠈⠙⢷⡄ ⠁ ⢀⢄⣴⣴⢵⠿⢟⣭⣽⠟⠁","            ⢉⢙⣛⣿⠷⢤⣄⡀  ⢿⢠⠔⢓⣛⠿⠿⠷⠷⠟⠛⠉⠁","            ⠈⠙⠛⠛⠓⠒⢖⠙⠷⡄⢈⡴⠟⢅⢴⣶⣤⣤⠄","                   ⢷⡄⠱⢍⡤⠐⠉⠉⠉⠉","                   ⠈⣷ ⠍","                    ⣿⢄"]

export const SHADE: string[] = ["            7767687656668    66","         678755455455555564566796","      6757654566556666656555433478","   687536652366 665666665565563 476675","3887663 465  65 56666666666666045563479","8656642  565 5666  666  656536566  35477"," 835643   4566  6665666566655556   45367"," 7435643  22266 66666653   0026  335556"," 3866545555    26033454333300 2344555575","    666665655662  03222300 6565566 24468","       6755 356655555   66563 0 34347886","         776543    3656 3 33444344686","            76765553  5335555556780","            6965564445565346896","                   5532433565","                   36 2","                    62"]
