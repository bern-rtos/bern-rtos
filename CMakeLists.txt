cmake_minimum_required(VERSION 3.17)
project("bern-rtos")


add_custom_target(
        BuildF446Dev
        COMMAND cd  ${CMAKE_CURRENT_SOURCE_DIR}/platform/st_nucleo_f446 && DEFMT_LOG=info cargo build --example=nucleo_f446_dev
        #BYPRODUCTS ${CMAKE_CURRENT_SOURCE_DIR}/target/thumbv7em-none-eabihf/debug/st_nucleo_f446re
)

add_custom_target(
        BuildF446Defmt
        COMMAND cd  ${CMAKE_CURRENT_SOURCE_DIR}/platform/st_nucleo_f446 && DEFMT_LOG=info cargo build --example=nucleo_f446_defmt
)

#add_custom_target(
#        BuildRelease
#        COMMAND cargo build --example=nucleo_f446_dev --release
#        #BYPRODUCTS ${CMAKE_CURRENT_SOURCE_DIR}/target/thumbv7em-none-eabihf/release/st_nucleo_f446re
#)