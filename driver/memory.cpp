#include "memory.hpp"

ULONG memory::get_module_base(PEPROCESS process, LPWSTR module_name)
{
	if (!process) { return 0; }

	__try
	{
		PPEB32 peb32 = (PPEB32)PsGetProcessWow64Process(process);
		if (!peb32 || !peb32->Ldr) { return 0; }

		for (PLIST_ENTRY32 plist_entry = (PLIST_ENTRY32)((PPEB_LDR_DATA32)peb32->Ldr)->InLoadOrderModuleList.Flink;
			plist_entry != &((PPEB_LDR_DATA32)peb32->Ldr)->InLoadOrderModuleList;
			plist_entry = (PLIST_ENTRY32)plist_entry->Flink)
		{
			PLDR_DATA_TABLE_ENTRY32 pentry = CONTAINING_RECORD(plist_entry, LDR_DATA_TABLE_ENTRY32, InLoadOrderLinks);

			if (wcscmp((PWCH)pentry->BaseDllName.Buffer, module_name) == 0)
			{
				return pentry->DllBase;
			}
		}
	}
	__except (EXCEPTION_EXECUTE_HANDLER)
	{

	}

	return 0;
}

memory::module_info memory::get_module_info_32(PEPROCESS target_process, LPCWSTR module_name)
{
	if (!target_process) { return memory::module_info{ 0 }; }

	KAPC_STATE apc;
	KeStackAttachProcess(target_process, &apc);

	__try
	{
		PPEB32 peb32 = (PPEB32)PsGetProcessWow64Process(target_process);
		if (!peb32 || !peb32->Ldr) { return memory::module_info{ 0 }; }

		for (PLIST_ENTRY32 plist_entry = (PLIST_ENTRY32)((PPEB_LDR_DATA32)peb32->Ldr)->InLoadOrderModuleList.Flink;
			plist_entry != &((PPEB_LDR_DATA32)peb32->Ldr)->InLoadOrderModuleList;
			plist_entry = (PLIST_ENTRY32)plist_entry->Flink)
		{
			PLDR_DATA_TABLE_ENTRY32 pentry = CONTAINING_RECORD(plist_entry, LDR_DATA_TABLE_ENTRY32, InLoadOrderLinks);

			if (wcscmp((PWCH)pentry->BaseDllName.Buffer, module_name) == 0)
			{
				return memory::module_info{
					pentry->DllBase,
					pentry->SizeOfImage
				};
			}
		}
	}
	__except (EXCEPTION_EXECUTE_HANDLER)
	{
		KeUnstackDetachProcess(&apc);
		return memory::module_info{ 0 };
	}

	KeUnstackDetachProcess(&apc);
	return memory::module_info{ 0 };
}

memory::module_info memory::get_module_info_64(PEPROCESS target_process, LPCWSTR module_name)
{
	__try {
		PPEB pPeb = PsGetProcessPeb(target_process);

		if (!pPeb) {
			return memory::module_info{ 0 }; // failed
		}

		PPEB_LDR_DATA pLdr = (PPEB_LDR_DATA)pPeb->Ldr;

		if (!pLdr) {
			return memory::module_info{ 0 }; // failed
		}

		// loop the linked list
		for (PLIST_ENTRY list = (PLIST_ENTRY)pLdr->ModuleListLoadOrder.Flink;
			list != &pLdr->ModuleListLoadOrder; list = (PLIST_ENTRY)list->Flink) {
			PLDR_DATA_TABLE_ENTRY pEntry =
				CONTAINING_RECORD(list, LDR_DATA_TABLE_ENTRY, InLoadOrderModuleList);

			auto len = min(wcslen(module_name), pEntry->BaseDllName.Length);
			//-DbgPrintEx(0, 0, "Found: %wZ, len %u, module_name: %S\n", pEntry->BaseDllName, len, module_name);
			if (!wcsncmp(pEntry->BaseDllName.Buffer, module_name, len))
			{
				ULONG64 baseAddr = (ULONG64)pEntry->DllBase;
				return memory::module_info{ (UINT64)pEntry->DllBase, pEntry->SizeOfImage };
			}
		}
	}
	__except(EXCEPTION_EXECUTE_HANDLER) {
		return memory::module_info{ 0 };
	}
	return memory::module_info{ 0 };
}

NTSTATUS memory::read_memory(PEPROCESS target_process, void* source, void* target, size_t size)
{
	if (!target_process) { return STATUS_INVALID_PARAMETER; }

	SIZE_T bytes = 0;
	//-DbgPrintEx(0, 0, "Reading %d bytes of memory into %p\n", bytes, target);
	NTSTATUS status = MmCopyVirtualMemory(target_process, source, IoGetCurrentProcess(), target, size, KernelMode, &bytes);
	if (!NT_SUCCESS(status) || !bytes)
	{
		//-DbgPrintEx(0, 0, "Not successful (%d bytes read, status %lx)\n", bytes, status);
		return STATUS_INVALID_ADDRESS;
	}

	return status;
}

NTSTATUS memory::write_memory(PEPROCESS target_process, void* source, void* target, size_t size)
{
	if (!target_process) { return STATUS_INVALID_PARAMETER; }

	SIZE_T bytes = 0;
	NTSTATUS status = MmCopyVirtualMemory(IoGetCurrentProcess(), source, target_process, target, size, KernelMode, &bytes);
	if (!NT_SUCCESS(status) || !bytes)
	{
		return STATUS_INVALID_ADDRESS;
	}
	return status;
}
