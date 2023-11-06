cmake_minimum_required(VERSION 3.27)
project(picochat)

file(GLOB sourcecode
  ${PROJECT_SOURCE_DIR}/include/*.h
  ${PROJECT_SOURCE_DIR}/src/*.c
)

add_executable(${PROJECT_NAME} ${sourcecode})