# pbs-rs
A Physically based shading demo app written in Rust using OpenGL 4.6 latest features

# Features

## Rendering
* [x] Physically based BRDF (Cook-Torrance)
* [x] Image Based Lighting
* [x] HDR rendering
* [x] ACES Tonemapping
* [x] Bloom
* [ ] Antialiasing
* [ ] SSAO
* [ ] SSR

# Samples

| Indoors ACES | Indoors ACES (simple luminance fit) |
|:-:|:-:|
| <img src="images/pbs-rs-aces-fitted-indoors.png"> | <img src="images/pbs-rs-aces-filmic-indoors.png"> |

| Outdoors ACES | Outdoors ACES (simple luminance fit) |
|:-:|:-:|
| <img src="images/pbs-rs-aces-fitted.png"> | <img src="images/pbs-rs-aces-filmic.png"> |

# References
* https://learnopengl.com/
* https://knarkowicz.wordpress.com/2016/01/06/aces-filmic-tone-mapping-curve/
