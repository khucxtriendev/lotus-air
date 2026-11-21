#include <ntddk.h>
#include <wdf.h>
#include <bthdef.h>
#include <bthioctl.h>
#include "ioctl.h"

#define LOTUSAIR_DEVICE_NAME L"\\Device\\LotusAirL2CAP"
#define LOTUSAIR_DOS_DEVICE_NAME L"\\DosDevices\\LotusAirL2CAP"

EVT_WDF_DRIVER_DEVICE_ADD LotusAirEvtDeviceAdd;
EVT_WDF_IO_QUEUE_IO_DEVICE_CONTROL LotusAirEvtIoDeviceControl;

NTSTATUS DriverEntry(
    _In_ PDRIVER_OBJECT DriverObject,
    _In_ PUNICODE_STRING RegistryPath
) {
    WDF_DRIVER_CONFIG config;
    NTSTATUS status;

    KdPrint(("LotusAir: DriverEntry bat dau khoi tao.\n"));

    WDF_DRIVER_CONFIG_INIT(&config, LotusAirEvtDeviceAdd);

    status = WdfDriverCreate(
        DriverObject,
        RegistryPath,
        WDF_NO_OBJECT_ATTRIBUTES,
        &config,
        WDF_NO_HANDLE
    );

    if (!NT_SUCCESS(status)) {
        KdPrint(("LotusAir: WdfDriverCreate that bai: 0x%08X\n", status));
    }

    return status;
}

NTSTATUS LotusAirEvtDeviceAdd(
    _In_ WDFDRIVER Driver,
    _Inout_ PWDFDEVICE_INIT DeviceInit
) {
    NTSTATUS status;
    WDFDEVICE device;
    WDF_IO_QUEUE_CONFIG queueConfig;
    UNICODE_STRING deviceName;
    UNICODE_STRING dosDeviceName;

    UNREFERENCED_PARAMETER(Driver);

    RtlInitUnicodeString(&deviceName, LOTUSAIR_DEVICE_NAME);
    status = WdfDeviceInitAssignName(DeviceInit, &deviceName);
    if (!NT_SUCCESS(status)) return status;

    status = WdfDeviceCreate(&DeviceInit, WDF_NO_OBJECT_ATTRIBUTES, &device);
    if (!NT_SUCCESS(status)) return status;

    RtlInitUnicodeString(&dosDeviceName, LOTUSAIR_DOS_DEVICE_NAME);
    status = WdfDeviceCreateSymbolicLink(device, &dosDeviceName);
    if (!NT_SUCCESS(status)) return status;

    WDF_IO_QUEUE_CONFIG_INIT_DEFAULT_QUEUE(&queueConfig, WdfIoQueueDispatchSequential);
    queueConfig.EvtIoDeviceControl = LotusAirEvtIoDeviceControl;

    status = WdfIoQueueCreate(device, &queueConfig, WDF_NO_OBJECT_ATTRIBUTES, WDF_NO_HANDLE);
    return status;
}

VOID LotusAirEvtIoDeviceControl(
    _In_ WDFQUEUE Queue,
    _In_ WDFREQUEST Request,
    _In_ size_t OutputBufferLength,
    _In_ size_t InputBufferLength,
    _In_ ULONG IoControlCode
) {
    NTSTATUS status = STATUS_SUCCESS;
    size_t bytesReturned = 0;

    UNREFERENCED_PARAMETER(Queue);
    UNREFERENCED_PARAMETER(OutputBufferLength);
    UNREFERENCED_PARAMETER(InputBufferLength);

    switch (IoControlCode) {
        case IOCTL_LOTUSAIR_CONNECT: {
            KdPrint(("LotusAir: Xu ly IOCTL_LOTUSAIR_CONNECT L2CAP PSM 0x1001\n"));
            bytesReturned = 0;
            break;
        }
        case IOCTL_LOTUSAIR_SEND: {
            KdPrint(("LotusAir: Gui goi tin AAP toi AirPods qua L2CAP\n"));
            bytesReturned = 0;
            break;
        }
        case IOCTL_LOTUSAIR_RECV: {
            KdPrint(("LotusAir: Doc goi tin AAP tu AirPods\n"));
            bytesReturned = 0;
            break;
        }
        case IOCTL_LOTUSAIR_DISCONNECT: {
            KdPrint(("LotusAir: Dong kenh L2CAP voi AirPods\n"));
            bytesReturned = 0;
            break;
        }
        default:
            status = STATUS_INVALID_DEVICE_REQUEST;
            break;
    }

    WdfRequestCompleteWithInformation(Request, status, bytesReturned);
}
