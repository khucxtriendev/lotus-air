#pragma once

#ifndef CTL_CODE
#define FILE_DEVICE_UNKNOWN 0x00000022
#define METHOD_BUFFERED 0
#define FILE_ANY_ACCESS 0
#define CTL_CODE(DeviceType, Function, Method, Access) ( \
    ((DeviceType) << 16) | ((Access) << 14) | ((Function) << 2) | (Method) \
)
#endif

#define LOTUSAIR_DEVICE_TYPE 0x00000022

#define IOCTL_LOTUSAIR_CONNECT \
    CTL_CODE(LOTUSAIR_DEVICE_TYPE, 0x800, METHOD_BUFFERED, FILE_ANY_ACCESS)

#define IOCTL_LOTUSAIR_SEND \
    CTL_CODE(LOTUSAIR_DEVICE_TYPE, 0x801, METHOD_BUFFERED, FILE_ANY_ACCESS)

#define IOCTL_LOTUSAIR_RECV \
    CTL_CODE(LOTUSAIR_DEVICE_TYPE, 0x802, METHOD_BUFFERED, FILE_ANY_ACCESS)

#define IOCTL_LOTUSAIR_DISCONNECT \
    CTL_CODE(LOTUSAIR_DEVICE_TYPE, 0x803, METHOD_BUFFERED, FILE_ANY_ACCESS)

#pragma pack(push, 1)
typedef struct _LOTUSAIR_CONNECT_REQ {
    unsigned long long BluetoothAddress;
    unsigned short Psm; // 0x1001 for AAP
    unsigned short TargetDevicePath[260]; // e.g. L"\\??\\BTHENUM#Dev_...#{00f40965-...}"
} LOTUSAIR_CONNECT_REQ, *PLOTUSAIR_CONNECT_REQ;
#pragma pack(pop)
