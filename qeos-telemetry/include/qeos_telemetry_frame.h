/*
 * qeos_telemetry_frame.h — Shared telemetry frame layout (kernel/userspace)
 *
 * [Production Kernel Component]
 *
 * Fixed-size, C-compatible, 64-bit aligned. Must match Rust EnergyTelemetryFrame.
 * Total size: 24 bytes.
 */

#ifndef QEOS_TELEMETRY_FRAME_H
#define QEOS_TELEMETRY_FRAME_H

#include <linux/types.h>

#define QEOS_FRAME_SIZE           24
#define QEOS_CACHE_LINE_SIZE      64
#define QEOS_TELEMETRY_DEVICE     "qeos-telemetry0"
#define QEOS_TELEMETRY_CLASS      "qeos_telemetry"

/* Frame flags (bitmask) */
#define QEOS_FLAG_OVERVOLTAGE      (1u << 0)
#define QEOS_FLAG_OVERCURRENT      (1u << 1)
#define QEOS_FLAG_FREQ_ANOMALY     (1u << 2)
#define QEOS_FLAG_CALIBRATED       (1u << 3)
#define QEOS_FLAG_IRQ_DRIVEN       (1u << 4)
#define QEOS_FLAG_DMA_VALID        (1u << 5)
#define QEOS_FLAG_CHECKSUM_OK      (1u << 6)
#define QEOS_FLAG_ANOMALY          (1u << 7)
#define QEOS_FLAG_GRID_INSTABLE    (1u << 8)
#define QEOS_FLAG_PRIORITY_HIGH    (1u << 9)

struct qeos_energy_telemetry_frame {
	__u64 timestamp_ns;
	__u32 sensor_id;
	__u32 voltage_mv;
	__u32 current_ma;
	__u16 frequency_hz_x100;
	__u16 flags;
} __attribute__((packed));

_Static_assert(sizeof(struct qeos_energy_telemetry_frame) == QEOS_FRAME_SIZE,
	       "frame size must be 24 bytes");

/* mmap metadata region (follows DMA frame array) */
struct qeos_telemetry_mmap_meta {
	__u64 head;       /* producer index (frames) */
	__u64 tail;       /* consumer index (frames) */
	__u64 capacity;   /* ring capacity (power of 2) */
	__u64 flags;      /* runtime flags */
	__u64 dropped;    /* atomic counter mirror */
	__u64 overwritten;
	__u64 irq_count;
	__u64 mode;       /* backpressure mode */
	__u64 reserved[5];
} __attribute__((aligned(QEOS_CACHE_LINE_SIZE)));

#define QEOS_MMAP_META_SIZE sizeof(struct qeos_telemetry_mmap_meta)

/* Hardware MMIO register block (simulated or FPGA) */
struct qeos_hw_regs {
	__u32 isr;        /* interrupt status */
	__u32 imr;        /* interrupt mask */
	__u32 dma_sr;     /* DMA status */
	__u32 dma_cr;     /* DMA control */
	__u64 dma_src;    /* bus source address */
	__u64 dma_dst;    /* bus destination (DMA buffer) */
	__u32 dma_size;   /* transfer size bytes */
	__u32 sensor_id;
	__u64 timestamp_ns;
	__u32 frame_count;
	__u32 error_sr;
};

#define QEOS_ISR_DATA_READY  0x1
#define QEOS_ISR_DMA_DONE    0x2
#define QEOS_ISR_ERROR       0x4

#endif /* QEOS_TELEMETRY_FRAME_H */
