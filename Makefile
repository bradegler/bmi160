.DEFAULT_GOAL 	 := build
PROJECT_NAME     := bmi160

.PHONY: clean build

build:
	@cargo build

clean:
	@cargo clean
