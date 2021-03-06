#!/bin/bash
#
#  SOS: the Stupid Operating System
#  by Eliza Weisman (hi@hawkweisman.me)
#
#  Copyright (c) 2015 Eliza Weisman
#  Released under the terms of the MIT license. See `LICENSE` in the root
#  directory of this repository for more information.
#
#  Portions of this script are adapted from a script by Steve Klabnik for
#  the IntermezzOS project, available at
#  http://intermezzos.github.io/book/appendix/osx-install.html
#
set -e
# this script will install the required dependencies and tools
# to build the SOS kernel.
bold=$(tput bold)
normal=$(tput sgr0)
# check if `brew` is installed
command -v brew >/dev/null 2>&1
if [ $? -eq 1 ]
then
    echo "${bold}install-mac:${normal} Homebrew is not installed."
    echo "${bold}install-mac:${normal} Please go to http://brew.sh/ to install it before continuing."
    exit
fi

export PREFIX="$HOME/opt/"
export TARGET=x86_64-pc-elf
export PATH="$PREFIX/bin:$PATH"

mkdir -p $HOME/src
mkdir -p $PREFIX

# dependencies installable with brew
echo "${bold}install-mac:${normal} Installing dependencies using Homebrew..."
brew update | sed "s/^/${bold}brew:${normal} /"
brew tap Homebrew/bundle | sed "s/^/${bold}brew:${normal} /"
brew bundle | sed "s/^/${bold}brew:${normal} /"

# binutils

cd $HOME/src

if [ ! -d "binutils-2.25" ]; then
    echo ""
    echo "${bold}install-mac:${normal} Installing GNU \`binutils\`"
    echo ""
    curl http://ftp.gnu.org/gnu/binutils/binutils-2.25.tar.gz > binutils-2.25.tar.gz
    tar xfz binutils-2.25.tar.gz
    rm binutils-2.25.tar.gz
    mkdir -p build-binutils
    cd build-binutils
    ../binutils-2.25/configure --target=$TARGET --prefix="$PREFIX" --with-sysroot --disable-nls --disable-werror
    make
    make install
else
    echo "${bold}install-mac:${normal} GNU \`binutils\` v2.25 is already installed, skipping."
fi

# gcc
cd $HOME/src

if [ ! -d "gcc-5.3.0" ]; then
  echo ""
  echo "${bold}install-mac:${normal} Installing \`gcc\`..."
  echo ""
  curl -L http://ftpmirror.gnu.org/gcc/gcc-5.3.0/gcc-5.3.0.tar.bz2 > gcc-5.3.0.tar.bz2
  tar jxf gcc-5.3.0.tar.bz2
  rm gcc-5.3.0.tar.bz2
  mkdir -p build-gcc
  cd build-gcc
  ../gcc-5.3.0/configure --target=$TARGET --prefix="$PREFIX" --disable-nls --enable-languages=c,c++ --without-headers --with-gmp=/usr/local/Cellar/gmp/6.1.0 --with-mpfr=/usr/local/Cellar/mpfr/3.1.3 --with-mpc=/usr/local/Cellar/libmpc/1.0.3
  make all-gcc
  make all-target-libgcc
  make install-gcc
  make install-target-libgcc
else
    echo "${bold}install-mac:${normal}  \`gcc\` v5.3.0 is already installed, skipping."
fi

# objconv

cd $HOME/src

if [ ! -d "objconv" ]; then
  echo "${bold}install-mac:${normal} Installing \`objconv\`..."
  curl http://www.agner.org/optimize/objconv.zip > objconv.zip
  mkdir -p build-objconv
  unzip objconv.zip -d build-objconv
  cd build-objconv
  unzip source.zip -d src
  g++ -o objconv -O2 src/*.cpp --prefix="$PREFIX"
  cp objconv $PREFIX/bin
else
    echo "${bold}install-mac:${normal} \`objconv\` is already installed, skipping."
fi

# grub

cd $HOME/src

if [ ! -d "grub" ]; then
  echo ""
  echo "${bold}install-mac:${normal} Installing \`grub\`..."
  echo ""
  git clone --depth 1 git://git.savannah.gnu.org/grub.git
  cd grub
  sh autogen.sh
  mkdir -p build-grub
  cd build-grub
  ../configure --disable-werror TARGET_CC=$TARGET-gcc TARGET_OBJCOPY=$TARGET-objcopy \
    TARGET_STRIP=$TARGET-strip TARGET_NM=$TARGET-nm TARGET_RANLIB=$TARGET-ranlib --target=$TARGET --prefix=$PREFIX
  make
  make install
else
    echo "${bold}install-mac:${normal} \`grub\` is already installed, skipping."
fi

CARGO_CONFIG="$HOME/.cargo/config"
GREP_TARGET_LINKER="\[target\.x86_64\-sos\-kernel\-gnu\]"
TARGET_LINKER="[[target.x86-sos-kernel-gnu]]\nlinker = \"${PREFIX}bin/x86_64-pc-elf-gcc\""

if grep -q $GREP_TARGET_LINKER "$CARGO_CONFIG"; then
    echo "${bold}install-mac:${normal} Target linker already present in $CARGO_CONFIG. Done."
else
    echo "${bold}install-mac:${normal} Adding target linker to $CARGO_CONFIG..."
    echo -e "\n" >> "$CARGO_CONFIG"
    echo -e "$TARGET_LINKER" >> "$CARGO_CONFIG"
fi
