# Fortran compiler and flags
FC 		= gfortran
FFLAGS 	= -O3 -fopenmp -march=native -fPIC -std=f2018
LDFLAGS = -shared -fopenmp

# OpenBLAS
BLAS_LIBS = -lopenblas

# Output
LIB = libwvec_core.so

# Source files (order matters for dependencies)
SRCS = wvec_types.f90 		\
	 wvec_blas.f90 			\
	 wvec_thermal.f90 		\
	 wvec_model.f90 		\
	 wvec_checkpoint.f90 	\
	 wvec_train.f90

# Object files
OBJS = $(SRCS:.f90=.o)

# Default target
all: $(LIB)

# Link shared library
$(LIB): $(OBJS)
	$(FC) $(LDFLAGS) -o $@ $^ $(BLAS_LIBS)

# Compile .f90 to .o
%.o: %.f90
	$(FC) $(FFLAGS) -c $< -o $@

# Dependencies (modules must be compiled before files that use them)
wvec_model.o: wvec_types.o
wvec_checkpoint.o: wvec_types.o wvec_model.o
wvec_train.o: wvec_types.o wvec_blas.o wvec_model.o

# Clean
clean:
	rm -f *.o *.mod $(LIB)

.PHONY: all clean
