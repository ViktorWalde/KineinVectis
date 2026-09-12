# Fixture MINIMA de um projeto ESP-IDF, como o `idf.py create-project` gera:
# a linha que decide e' o include do project.cmake pelo IDF_PATH.
cmake_minimum_required(VERSION 3.16)
include($ENV{IDF_PATH}/tools/cmake/project.cmake)
project(fixture_esp_idf)
