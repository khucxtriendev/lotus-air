/*
 * LotusAirDriver.sys - WDM Kernel-Mode Bluetooth L2CAP Bridge (PSM 0x1001)
 * Giao tiếp trực tiếp với Windows bthport.sys / bthenum.sys qua IOCTL_INTERNAL_BTH_SUBMIT_BRB (0x00410003)
 */

#include "ioctl.h"

typedef long NTSTATUS;
typedef unsigned char UCHAR;
typedef unsigned short USHORT;
typedef unsigned short WCHAR;
typedef unsigned int ULONG;
typedef unsigned long long ULONGLONG;
typedef long long LONGLONG;
typedef unsigned long long ULONG_PTR;
typedef unsigned long long SIZE_T;
typedef void *PVOID;
typedef char CCHAR;
typedef short CSHORT;
typedef UCHAR BOOLEAN;

#define NTAPI __stdcall
#define TRUE 1
#define FALSE 0
#define NULL ((void *)0)

#define STATUS_SUCCESS ((NTSTATUS)0x00000000L)
#define STATUS_PENDING ((NTSTATUS)0x00000103L)
#define STATUS_MORE_PROCESSING_REQUIRED ((NTSTATUS)0xC0000016L)
#define STATUS_INVALID_PARAMETER ((NTSTATUS)0xC000000DL)
#define STATUS_INVALID_DEVICE_REQUEST ((NTSTATUS)0xC0000010L)
#define STATUS_BUFFER_TOO_SMALL ((NTSTATUS)0xC0000023L)
#define STATUS_INSUFFICIENT_RESOURCES ((NTSTATUS)0xC000009AL)
#define STATUS_DEVICE_NOT_CONNECTED ((NTSTATUS)0xC000009DL)
#define STATUS_INVALID_BUFFER_SIZE ((NTSTATUS)0xC0000206L)
#define NT_SUCCESS(Status) (((NTSTATUS)(Status)) >= 0)

#define IRP_MJ_CREATE 0x00
#define IRP_MJ_CLOSE 0x02
#define IRP_MJ_DEVICE_CONTROL 0x0E
#define IRP_MJ_INTERNAL_DEVICE_CONTROL 0x0F
#define IRP_MJ_MAXIMUM_FUNCTION 0x1B

#define IO_NO_INCREMENT 0
#define KernelMode 0
#define Executive 0
#define NotificationEvent 0
#define POOL_FLAG_NON_PAGED 0x0000000000000040ULL
#define LOTUSAIR_POOL_TAG 0x7241744CUL /* 'LtAr' */

#define IOCTL_INTERNAL_BTH_SUBMIT_BRB 0x00410003UL

/* BRB Types (verified from bthport.sys 10.0.26100) */
#define BRB_L2CA_OPEN_CHANNEL 0x0102
#define BRB_L2CA_CLOSE_CHANNEL 0x0104
#define BRB_L2CA_ACL_TRANSFER 0x0105

#define BRB_L2CA_OPEN_CHANNEL_SIZE 0x0170UL
#define BRB_L2CA_CLOSE_CHANNEL_SIZE 0x0080UL
#define BRB_L2CA_ACL_TRANSFER_SIZE 0x00A8UL

#define CF_ROLE_EITHER 0x00000001UL
#define ACL_TRANSFER_DIRECTION_OUT 0x00000000UL
#define ACL_TRANSFER_DIRECTION_IN 0x00000001UL
#define ACL_SHORT_TRANSFER_OK 0x00000002UL
#define ACL_TRANSFER_TIMEOUT 0x00000004UL

typedef struct _UNICODE_STRING {
    USHORT Length;
    USHORT MaximumLength;
    WCHAR *Buffer;
} UNICODE_STRING, *PUNICODE_STRING;

typedef struct _LIST_ENTRY {
    struct _LIST_ENTRY *Flink;
    struct _LIST_ENTRY *Blink;
} LIST_ENTRY, *PLIST_ENTRY;

typedef struct _DISPATCHER_HEADER {
    UCHAR Type;
    UCHAR Signalling;
    UCHAR Size;
    UCHAR Reserved1;
    long SignalState;
    LIST_ENTRY WaitListHead;
} DISPATCHER_HEADER;

typedef struct _KEVENT {
    DISPATCHER_HEADER Header;
} KEVENT, *PKEVENT;

typedef struct _IO_STATUS_BLOCK {
    union {
        NTSTATUS Status;
        PVOID Pointer;
    };
    ULONG_PTR Information;
} IO_STATUS_BLOCK, *PIO_STATUS_BLOCK;

struct _DEVICE_OBJECT;
struct _DRIVER_OBJECT;
struct _IRP;
struct _FILE_OBJECT;

typedef NTSTATUS (NTAPI *PIO_COMPLETION_ROUTINE)(
    struct _DEVICE_OBJECT *DeviceObject,
    struct _IRP *Irp,
    PVOID Context
);

typedef struct _IO_STACK_LOCATION {
    UCHAR MajorFunction;
    UCHAR MinorFunction;
    UCHAR Flags;
    UCHAR Control;
    union {
        struct {
            ULONG OutputBufferLength;
            ULONG Alignment1;
            ULONG InputBufferLength;
            ULONG Alignment2;
            ULONG IoControlCode;
            ULONG Alignment3;
            PVOID Type3InputBuffer;
        } DeviceIoControl;
        struct {
            PVOID Argument1;
            PVOID Argument2;
            PVOID Argument3;
            PVOID Argument4;
        } Others;
    } Parameters;
    struct _DEVICE_OBJECT *DeviceObject;
    struct _FILE_OBJECT *FileObject;
    PIO_COMPLETION_ROUTINE CompletionRoutine;
    PVOID Context;
} IO_STACK_LOCATION, *PIO_STACK_LOCATION;

typedef struct _IRP {
    CSHORT Type;
    USHORT Size;
    PVOID MdlAddress;
    ULONG Flags;
    union {
        struct _IRP *MasterIrp;
        long IrpCount;
        PVOID SystemBuffer;
    } AssociatedIrp;
    LIST_ENTRY ThreadListEntry;
    IO_STATUS_BLOCK IoStatus;
    CCHAR RequestorMode;
    BOOLEAN PendingReturned;
    CCHAR StackCount;
    CCHAR CurrentLocation;
    BOOLEAN Cancel;
    UCHAR CancelIrql;
    CCHAR ApcEnvironment;
    UCHAR AllocationFlags;
    PIO_STATUS_BLOCK UserIosb;
    PKEVENT UserEvent;
    union {
        struct {
            PVOID UserApcRoutine;
            PVOID UserApcContext;
        } AsynchronousParameters;
        LONGLONG AllocationSize;
    } Overlay;
    PVOID CancelRoutine;
    PVOID UserBuffer;
    union {
        struct {
            union {
                PVOID DeviceQueueEntry[4];
                struct {
                    PVOID DriverContext[4];
                };
            };
            PVOID Thread;
            char *AuxiliaryBuffer;
            LIST_ENTRY ListEntry;
            union {
                PIO_STACK_LOCATION CurrentStackLocation;
                ULONG PacketType;
            };
            struct _FILE_OBJECT *OriginalFileObject;
        } Overlay;
        UCHAR Apc[0x58];
        PVOID CompletionKey;
    } Tail;
} IRP, *PIRP;

typedef struct _DEVICE_OBJECT {
    CSHORT Type;
    USHORT Size;
    long ReferenceCount;
    struct _DRIVER_OBJECT *DriverObject;
    struct _DEVICE_OBJECT *NextDevice;
    struct _DEVICE_OBJECT *AttachedDevice;
    PIRP CurrentIrp;
    PVOID Timer;
    ULONG Flags;
    ULONG Characteristics;
    PVOID Vpb;
    PVOID DeviceExtension;
    ULONG DeviceType;
    CCHAR StackSize;
    UCHAR Padding[0x100];
} DEVICE_OBJECT, *PDEVICE_OBJECT;

typedef NTSTATUS (NTAPI *PDRIVER_DISPATCH)(PDEVICE_OBJECT DeviceObject, PIRP Irp);
typedef void (NTAPI *PDRIVER_UNLOAD)(struct _DRIVER_OBJECT *DriverObject);

typedef struct _DRIVER_OBJECT {
    CSHORT Type;
    CSHORT Size;
    PDEVICE_OBJECT DeviceObject;
    ULONG Flags;
    PVOID DriverStart;
    ULONG DriverSize;
    PVOID DriverSection;
    PVOID DriverExtension;
    UNICODE_STRING DriverName;
    PUNICODE_STRING HardwareDatabase;
    PVOID FastIoDispatch;
    PVOID DriverInit;
    PVOID DriverStartIo;
    PDRIVER_UNLOAD DriverUnload;
    PDRIVER_DISPATCH MajorFunction[IRP_MJ_MAXIMUM_FUNCTION + 1];
} DRIVER_OBJECT, *PDRIVER_OBJECT;

/* Kernel Exports from ntoskrnl.exe */
__declspec(dllimport) void NTAPI RtlInitUnicodeString(PUNICODE_STRING DestinationString, const WCHAR *SourceString);
__declspec(dllimport) NTSTATUS NTAPI IoCreateDevice(
    PDRIVER_OBJECT DriverObject,
    ULONG DeviceExtensionSize,
    PUNICODE_STRING DeviceName,
    ULONG DeviceType,
    ULONG DeviceCharacteristics,
    BOOLEAN Exclusive,
    PDEVICE_OBJECT *DeviceObject
);
__declspec(dllimport) void NTAPI IoDeleteDevice(PDEVICE_OBJECT DeviceObject);
__declspec(dllimport) NTSTATUS NTAPI IoCreateSymbolicLink(PUNICODE_STRING SymbolicLinkName, PUNICODE_STRING DeviceName);
__declspec(dllimport) NTSTATUS NTAPI IoDeleteSymbolicLink(PUNICODE_STRING SymbolicLinkName);
__declspec(dllimport) NTSTATUS NTAPI IoGetDeviceObjectPointer(
    PUNICODE_STRING ObjectName,
    ULONG DesiredAccess,
    struct _FILE_OBJECT **FileObject,
    PDEVICE_OBJECT *DeviceObject
);
__declspec(dllimport) PIRP NTAPI IoAllocateIrp(CCHAR StackSize, BOOLEAN ChargeQuota);
__declspec(dllimport) void NTAPI IoFreeIrp(PIRP Irp);
__declspec(dllimport) NTSTATUS NTAPI IofCallDriver(PDEVICE_OBJECT DeviceObject, PIRP Irp);
__declspec(dllimport) void NTAPI IofCompleteRequest(PIRP Irp, CCHAR PriorityBoost);
__declspec(dllimport) void NTAPI KeInitializeEvent(PKEVENT Event, ULONG Type, BOOLEAN State);
__declspec(dllimport) long NTAPI KeSetEvent(PKEVENT Event, long Increment, BOOLEAN Wait);
__declspec(dllimport) NTSTATUS NTAPI KeWaitForSingleObject(
    PVOID Object,
    ULONG WaitReason,
    CCHAR WaitMode,
    BOOLEAN Alertable,
    LONGLONG *Timeout
);
__declspec(dllimport) PVOID NTAPI ExAllocatePool2(ULONGLONG Flags, SIZE_T NumberOfBytes, ULONG Tag);
__declspec(dllimport) void NTAPI ExFreePoolWithTag(PVOID P, ULONG Tag);
__declspec(dllimport) void NTAPI ObfDereferenceObject(PVOID Object);

typedef struct _LOTUSAIR_EXTENSION {
    struct _FILE_OBJECT *BthFileObject;
    PDEVICE_OBJECT BthDeviceObject;
    ULONGLONG RemoteAddress;
    USHORT Psm;
    PVOID ChannelHandle;
    BOOLEAN IsChannelOpen;
} LOTUSAIR_EXTENSION, *PLOTUSAIR_EXTENSION;

static void ZeroBuf(void *ptr, SIZE_T len) {
    UCHAR *p = (UCHAR *)ptr;
    SIZE_T i;
    for (i = 0; i < len; i++) {
        p[i] = 0;
    }
}

static void CopyBuf(void *dst, const void *src, SIZE_T len) {
    UCHAR *d = (UCHAR *)dst;
    const UCHAR *s = (const UCHAR *)src;
    SIZE_T i;
    for (i = 0; i < len; i++) {
        d[i] = s[i];
    }
}

static NTSTATUS NTAPI BrbSyncCompletion(
    PDEVICE_OBJECT DeviceObject,
    PIRP Irp,
    PVOID Context
) {
    PKEVENT event = (PKEVENT)Context;
    (void)DeviceObject;
    (void)Irp;
    if (event != NULL) {
        KeSetEvent(event, IO_NO_INCREMENT, FALSE);
    }
    return STATUS_MORE_PROCESSING_REQUIRED;
}

static NTSTATUS SubmitBrbSynchronous(
    PDEVICE_OBJECT TargetDevice,
    PVOID Brb
) {
    PIRP irp;
    PIO_STACK_LOCATION irpSp;
    KEVENT event;
    NTSTATUS status;

    if (TargetDevice == NULL || Brb == NULL) {
        return STATUS_DEVICE_NOT_CONNECTED;
    }

    irp = IoAllocateIrp((CCHAR)(TargetDevice->StackSize + 1), FALSE);
    if (irp == NULL) {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    KeInitializeEvent(&event, NotificationEvent, FALSE);

    /* Next stack location is CurrentStackLocation - 1 */
    irpSp = irp->Tail.Overlay.CurrentStackLocation - 1;
    ZeroBuf(irpSp, sizeof(IO_STACK_LOCATION));
    irpSp->MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL;
    irpSp->Parameters.DeviceIoControl.IoControlCode = IOCTL_INTERNAL_BTH_SUBMIT_BRB;
    irpSp->Parameters.Others.Argument1 = Brb;
    irpSp->CompletionRoutine = BrbSyncCompletion;
    irpSp->Context = &event;
    irpSp->Control = 0xE0; /* SL_INVOKE_ON_SUCCESS | SL_INVOKE_ON_ERROR | SL_INVOKE_ON_CANCEL */

    status = IofCallDriver(TargetDevice, irp);
    if (status == STATUS_PENDING) {
        KeWaitForSingleObject(&event, Executive, KernelMode, FALSE, NULL);
        status = irp->IoStatus.Status;
    }

    IoFreeIrp(irp);
    return status;
}

static void CloseCurrentChannel(PLOTUSAIR_EXTENSION ext) {
    UCHAR *brb;
    if (ext == NULL) {
        return;
    }
    if (ext->IsChannelOpen && ext->ChannelHandle != NULL && ext->BthDeviceObject != NULL) {
        brb = (UCHAR *)ExAllocatePool2(POOL_FLAG_NON_PAGED, BRB_L2CA_CLOSE_CHANNEL_SIZE, LOTUSAIR_POOL_TAG);
        if (brb != NULL) {
            ZeroBuf(brb, BRB_L2CA_CLOSE_CHANNEL_SIZE);
            *(ULONG *)(brb + 0x10) = BRB_L2CA_CLOSE_CHANNEL_SIZE;
            *(USHORT *)(brb + 0x16) = BRB_L2CA_CLOSE_CHANNEL;
            *(ULONGLONG *)(brb + 0x70) = ext->RemoteAddress;
            *(PVOID *)(brb + 0x78) = ext->ChannelHandle;
            SubmitBrbSynchronous(ext->BthDeviceObject, brb);
            ExFreePoolWithTag(brb, LOTUSAIR_POOL_TAG);
        }
    }
    ext->IsChannelOpen = FALSE;
    ext->ChannelHandle = NULL;
    if (ext->BthFileObject != NULL) {
        ObfDereferenceObject(ext->BthFileObject);
        ext->BthFileObject = NULL;
        ext->BthDeviceObject = NULL;
    }
}

static NTSTATUS OpenL2capChannel(
    PLOTUSAIR_EXTENSION ext,
    const LOTUSAIR_CONNECT_REQ *req
) {
    UNICODE_STRING targetName;
    UCHAR *brb;
    NTSTATUS status;

    CloseCurrentChannel(ext);

    if (req->TargetDevicePath[0] != 0) {
        RtlInitUnicodeString(&targetName, req->TargetDevicePath);
    } else {
        RtlInitUnicodeString(&targetName, L"\\Device\\BthPort");
    }

    status = IoGetDeviceObjectPointer(
        &targetName,
        FILE_ANY_ACCESS,
        &ext->BthFileObject,
        &ext->BthDeviceObject
    );
    if (!NT_SUCCESS(status)) {
        ext->BthFileObject = NULL;
        ext->BthDeviceObject = NULL;
        return status;
    }

    brb = (UCHAR *)ExAllocatePool2(POOL_FLAG_NON_PAGED, BRB_L2CA_OPEN_CHANNEL_SIZE, LOTUSAIR_POOL_TAG);
    if (brb == NULL) {
        CloseCurrentChannel(ext);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    ZeroBuf(brb, BRB_L2CA_OPEN_CHANNEL_SIZE);
    *(ULONG *)(brb + 0x10) = BRB_L2CA_OPEN_CHANNEL_SIZE;
    *(USHORT *)(brb + 0x16) = BRB_L2CA_OPEN_CHANNEL;
    *(PVOID *)(brb + 0x70) = NULL;                  /* ChannelHandle (output) */
    *(USHORT *)(brb + 0x78) = (req->Psm != 0) ? req->Psm : 0x1001;
    *(ULONG *)(brb + 0x7C) = 0;                     /* ChannelFlags = 0 */
    *(ULONGLONG *)(brb + 0x80) = req->BluetoothAddress; /* BtAddress (input at 0x80!) */

    status = SubmitBrbSynchronous(ext->BthDeviceObject, brb);
    if (NT_SUCCESS(status)) {
        ext->RemoteAddress = req->BluetoothAddress;
        ext->Psm = (req->Psm != 0) ? req->Psm : 0x1001;
        ext->ChannelHandle = *(PVOID *)(brb + 0x70); /* ChannelHandle returned at 0x70! */
        ext->IsChannelOpen = (ext->ChannelHandle != NULL) ? TRUE : FALSE;
    } else {
        CloseCurrentChannel(ext);
    }

    ExFreePoolWithTag(brb, LOTUSAIR_POOL_TAG);
    return status;
}

static NTSTATUS SendAclPacket(
    PLOTUSAIR_EXTENSION ext,
    const void *data,
    ULONG dataLen
) {
    UCHAR *brb;
    PVOID kbuf;
    NTSTATUS status;

    if (!ext->IsChannelOpen || ext->ChannelHandle == NULL || ext->BthDeviceObject == NULL) {
        return STATUS_DEVICE_NOT_CONNECTED;
    }

    brb = (UCHAR *)ExAllocatePool2(POOL_FLAG_NON_PAGED, BRB_L2CA_ACL_TRANSFER_SIZE, LOTUSAIR_POOL_TAG);
    if (brb == NULL) {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    kbuf = ExAllocatePool2(POOL_FLAG_NON_PAGED, dataLen, LOTUSAIR_POOL_TAG);
    if (kbuf == NULL) {
        ExFreePoolWithTag(brb, LOTUSAIR_POOL_TAG);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    CopyBuf(kbuf, data, dataLen);
    ZeroBuf(brb, BRB_L2CA_ACL_TRANSFER_SIZE);

    *(ULONG *)(brb + 0x10) = BRB_L2CA_ACL_TRANSFER_SIZE;
    *(USHORT *)(brb + 0x16) = BRB_L2CA_ACL_TRANSFER;
    *(ULONGLONG *)(brb + 0x70) = ext->RemoteAddress;
    *(PVOID *)(brb + 0x78) = ext->ChannelHandle;
    *(ULONG *)(brb + 0x80) = ACL_TRANSFER_DIRECTION_OUT;
    *(ULONG *)(brb + 0x84) = dataLen;
    *(PVOID *)(brb + 0x88) = kbuf;
    *(PVOID *)(brb + 0x90) = NULL;

    status = SubmitBrbSynchronous(ext->BthDeviceObject, brb);

    ExFreePoolWithTag(kbuf, LOTUSAIR_POOL_TAG);
    ExFreePoolWithTag(brb, LOTUSAIR_POOL_TAG);
    return status;
}

static NTSTATUS RecvAclPacket(
    PLOTUSAIR_EXTENSION ext,
    void *outBuf,
    ULONG outBufLen,
    ULONG_PTR *bytesRead
) {
    UCHAR *brb;
    PVOID kbuf;
    NTSTATUS status;
    ULONG remaining;

    *bytesRead = 0;

    if (!ext->IsChannelOpen || ext->ChannelHandle == NULL || ext->BthDeviceObject == NULL) {
        return STATUS_DEVICE_NOT_CONNECTED;
    }

    brb = (UCHAR *)ExAllocatePool2(POOL_FLAG_NON_PAGED, BRB_L2CA_ACL_TRANSFER_SIZE, LOTUSAIR_POOL_TAG);
    if (brb == NULL) {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    kbuf = ExAllocatePool2(POOL_FLAG_NON_PAGED, outBufLen, LOTUSAIR_POOL_TAG);
    if (kbuf == NULL) {
        ExFreePoolWithTag(brb, LOTUSAIR_POOL_TAG);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    ZeroBuf(kbuf, outBufLen);
    ZeroBuf(brb, BRB_L2CA_ACL_TRANSFER_SIZE);

    *(ULONG *)(brb + 0x10) = BRB_L2CA_ACL_TRANSFER_SIZE;
    *(USHORT *)(brb + 0x16) = BRB_L2CA_ACL_TRANSFER;
    *(ULONGLONG *)(brb + 0x70) = ext->RemoteAddress;
    *(PVOID *)(brb + 0x78) = ext->ChannelHandle;
    *(ULONG *)(brb + 0x80) = ACL_TRANSFER_DIRECTION_IN | ACL_SHORT_TRANSFER_OK | ACL_TRANSFER_TIMEOUT;
    *(ULONG *)(brb + 0x84) = outBufLen;
    *(PVOID *)(brb + 0x88) = kbuf;
    *(PVOID *)(brb + 0x90) = NULL;
    *(LONGLONG *)(brb + 0x98) = 50; /* 50ms non-blocking poll timeout */

    status = SubmitBrbSynchronous(ext->BthDeviceObject, brb);
    if (NT_SUCCESS(status)) {
        remaining = *(ULONG *)(brb + 0xA0);
        if (remaining < outBufLen) {
            *bytesRead = (ULONG_PTR)(outBufLen - remaining);
            CopyBuf(outBuf, kbuf, *bytesRead);
        }
    }

    ExFreePoolWithTag(kbuf, LOTUSAIR_POOL_TAG);
    ExFreePoolWithTag(brb, LOTUSAIR_POOL_TAG);
    return status;
}

static NTSTATUS NTAPI LotusAirCreateClose(PDEVICE_OBJECT DeviceObject, PIRP Irp) {
    (void)DeviceObject;
    Irp->IoStatus.Status = STATUS_SUCCESS;
    Irp->IoStatus.Information = 0;
    IofCompleteRequest(Irp, IO_NO_INCREMENT);
    return STATUS_SUCCESS;
}

static NTSTATUS NTAPI LotusAirDeviceControl(PDEVICE_OBJECT DeviceObject, PIRP Irp) {
    PLOTUSAIR_EXTENSION ext = (PLOTUSAIR_EXTENSION)DeviceObject->DeviceExtension;
    PIO_STACK_LOCATION irpSp = irpSp = Irp->Tail.Overlay.CurrentStackLocation;
    ULONG code = irpSp->Parameters.DeviceIoControl.IoControlCode;
    ULONG inLen = irpSp->Parameters.DeviceIoControl.InputBufferLength;
    ULONG outLen = irpSp->Parameters.DeviceIoControl.OutputBufferLength;
    PVOID sysBuf = Irp->AssociatedIrp.SystemBuffer;
    NTSTATUS status = STATUS_SUCCESS;
    ULONG_PTR info = 0;

    switch (code) {
        case IOCTL_LOTUSAIR_CONNECT: {
            LOTUSAIR_CONNECT_REQ req;
            ZeroBuf(&req, sizeof(req));
            if (inLen < 10 || sysBuf == NULL) {
                status = STATUS_BUFFER_TOO_SMALL;
                break;
            }
            CopyBuf(&req, sysBuf, (inLen < sizeof(req)) ? inLen : sizeof(req));
            status = OpenL2capChannel(ext, &req);
            break;
        }
        case IOCTL_LOTUSAIR_SEND: {
            if (inLen == 0 || inLen > 1024 || sysBuf == NULL) {
                status = STATUS_INVALID_BUFFER_SIZE;
                break;
            }
            status = SendAclPacket(ext, sysBuf, inLen);
            if (NT_SUCCESS(status)) {
                info = inLen;
            }
            break;
        }
        case IOCTL_LOTUSAIR_RECV: {
            if (outLen == 0 || sysBuf == NULL) {
                status = STATUS_BUFFER_TOO_SMALL;
                break;
            }
            status = RecvAclPacket(ext, sysBuf, (outLen < 1024) ? outLen : 1024, &info);
            break;
        }
        case IOCTL_LOTUSAIR_DISCONNECT: {
            CloseCurrentChannel(ext);
            status = STATUS_SUCCESS;
            break;
        }
        default:
            status = STATUS_INVALID_DEVICE_REQUEST;
            break;
    }

    Irp->IoStatus.Status = status;
    Irp->IoStatus.Information = info;
    IofCompleteRequest(Irp, IO_NO_INCREMENT);
    return status;
}

static void NTAPI LotusAirUnload(PDRIVER_OBJECT DriverObject) {
    UNICODE_STRING symLink;
    PDEVICE_OBJECT devObj = DriverObject->DeviceObject;
    if (devObj != NULL) {
        PLOTUSAIR_EXTENSION ext = (PLOTUSAIR_EXTENSION)devObj->DeviceExtension;
        CloseCurrentChannel(ext);
        RtlInitUnicodeString(&symLink, L"\\DosDevices\\LotusAirL2CAP");
        IoDeleteSymbolicLink(&symLink);
        IoDeleteDevice(devObj);
    }
}

NTSTATUS NTAPI DriverEntry(PDRIVER_OBJECT DriverObject, PUNICODE_STRING RegistryPath) {
    UNICODE_STRING devName;
    UNICODE_STRING symLink;
    PDEVICE_OBJECT devObj = NULL;
    NTSTATUS status;
    ULONG i;

    (void)RegistryPath;

    RtlInitUnicodeString(&devName, L"\\Device\\LotusAirL2CAP");
    status = IoCreateDevice(
        DriverObject,
        sizeof(LOTUSAIR_EXTENSION),
        &devName,
        FILE_DEVICE_UNKNOWN,
        0,
        FALSE,
        &devObj
    );
    if (!NT_SUCCESS(status)) {
        return status;
    }

    ZeroBuf(devObj->DeviceExtension, sizeof(LOTUSAIR_EXTENSION));

    RtlInitUnicodeString(&symLink, L"\\DosDevices\\LotusAirL2CAP");
    status = IoCreateSymbolicLink(&symLink, &devName);
    if (!NT_SUCCESS(status)) {
        IoDeleteDevice(devObj);
        return status;
    }

    for (i = 0; i <= IRP_MJ_MAXIMUM_FUNCTION; i++) {
        DriverObject->MajorFunction[i] = LotusAirCreateClose;
    }
    DriverObject->MajorFunction[IRP_MJ_DEVICE_CONTROL] = LotusAirDeviceControl;
    DriverObject->DriverUnload = LotusAirUnload;

    return STATUS_SUCCESS;
}
