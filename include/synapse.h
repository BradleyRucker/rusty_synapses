/*
 * Synapse Protocol - C/C++ FFI Header
 *
 * This header provides C-compatible bindings to the Rust synapse library.
 * Link with -lsynapse and ensure libsynapse.so/libsynapse.a is in your library path.
 */

#ifndef SYNAPSE_H
#define SYNAPSE_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Service IDs */
extern const uint8_t SYNAPSE_SERVICE_VIO;
extern const uint8_t SYNAPSE_STATUS_RUNNING;
extern const uint8_t SYNAPSE_STATUS_INIT;

/*
 * Build a Register message and write it to the output buffer.
 * Returns the number of bytes written, or -1 on error.
 */
int synapse_build_register(
    uint8_t service_id,
    uint32_t pid,
    const char* version,
    uint8_t* out_buf,
    size_t out_buf_len
);

/*
 * Build a ServiceHeartbeat message and write it to the output buffer.
 * Returns the number of bytes written, or -1 on error.
 */
int synapse_build_heartbeat(
    uint8_t service_id,
    uint8_t status,
    uint8_t* out_buf,
    size_t out_buf_len
);

/*
 * Build a VioPose message and write it to the output buffer.
 * Returns the number of bytes written, or -1 on error.
 */
int synapse_build_vio_pose(
    uint64_t timestamp_ns,
    double px, double py, double pz,
    double qw, double qx, double qy, double qz,
    double vx, double vy, double vz,
    bool initialized,
    uint8_t* out_buf,
    size_t out_buf_len
);

/*
 * Build a raw frame with given msg_id and payload.
 * Returns the number of bytes written, or -1 on error.
 */
int synapse_build_frame(
    uint16_t msg_id,
    const uint8_t* payload,
    size_t payload_len,
    uint8_t* out_buf,
    size_t out_buf_len
);

#ifdef __cplusplus
}
#endif

#endif /* SYNAPSE_H */
