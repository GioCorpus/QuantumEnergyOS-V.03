# QuantumEnergyOS telemetry kernel module build
#
# Usage (on Linux with kernel headers):
#   make -C /lib/modules/$(uname -r)/build M=$(PWD) modules
#   sudo insmod qeos_telemetry.ko
#   ls -l /dev/qeos-telemetry0

obj-m += qeos_telemetry.o

KDIR ?= /lib/modules/$(shell uname -r)/build
PWD  := $(shell pwd)

ccflags-y := -I$(src)/../include -Wall -Wextra

all:
	$(MAKE) -C $(KDIR) M=$(PWD) modules

clean:
	$(MAKE) -C $(KDIR) M=$(PWD) clean

install:
	$(MAKE) -C $(KDIR) M=$(PWD) modules_install

.PHONY: all clean install
