exe = 

###---Environment---###
CXX = g++
CXXFlags = -std=c++17 -I$(INCDIR) -Wall -pedantic

BINDIR = ./bin
INCDIR = ./inc
SRCDIR = ./src
OBJDIR = ./obj

SRC = $(wildcard $(SRCDIR)/*.cpp)
INC = $(wildcard $(INCDIR)/*.h)
OBJ = $(SRC:$(SRCDIR)/%.cpp=$(OBJDIR)/%.o)

###---Build---###
.PHONY: all
all: $(BINDIR)/$(exe)

$(BINDIR)/$(exe): $(OBJ)
	$(CXX) $(CXXFlags) -o $@ $(OBJ)

$(OBJDIR)/%.o: $(SRCDIR)/%.cpp
	$(CXX) $(CXXFlags) -o $@ -c $<

###---Utilities---###
.PHONY: clean, cleanwin
clean:
	rm -f $(BINDIR)/*.exe $(OBJDIR)/*.o

cleanwin:
	powershell -command "Remove-Item $(BINDIR)/*.exe; Remove-Item $(OBJDIR)/*.o"
