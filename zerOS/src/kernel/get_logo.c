#define STBI_NO_STDIO
#define STB_IMAGE_IMPLEMENTATION
#include <stb_image.h>

#define INCBIN_PREFIX zerOS_
#define INCBIN_STYLE INCBIN_STYLE_SNAKE
#include <incbin.h>

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

INCBIN(logo_white_transparent_svg, "assets/zeros-high-resolution-logo-white-transparent.svg");

void zerOS_get_logo(uint8_t** data, size_t* width, size_t* height, size_t* channels)
{
    int w, h, c;
    *data = stbi_load_from_memory(zerOS_logo_white_transparent_svg_data, zerOS_logo_white_transparent_svg_size, &w, &h, &c, 0);
    *width = w;
    *height = h;
    *channels = c;

    if (*data == NULL)
    {
        *width = 0;
        *height = 0;
        *channels = 0;
    }

    return;
}

