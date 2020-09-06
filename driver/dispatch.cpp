#include "dispatch.hpp"

void dispatch::handler(void* info_struct)
{
	DbgPrintEx(0, 0, "Hook was called with info struct %p\n", info_struct);
	PINFO_STRUCT info = (PINFO_STRUCT)info_struct;

	if (info->code == CODE_CLIENT_REQUEST)
	{
		PEPROCESS target_process = NULL;
		if (NT_SUCCESS(PsLookupProcessByProcessId((HANDLE)info->process_id, &target_process)))
		{
			KAPC_STATE apc;
			KeStackAttachProcess(target_process, &apc);
			ULONG b = memory::get_module_base(target_process, L"client.dll");
			KeUnstackDetachProcess(&apc);
			if (b) { info->client_base = b; }
		}
	}
	else if (info->code == CODE_READ_MEMORY)
	{
		DbgPrintEx(0, 0, "Reading %d bytes of memory from PID %d\n", info->size, info->process_id);
		PEPROCESS target_process = NULL;
		if (NT_SUCCESS(PsLookupProcessByProcessId((HANDLE)info->process_id, &target_process)))
		{
			memory::read_memory(target_process, (void*)info->address, (void*) info->buffer_addr, info->size);
			DbgPrintEx(0, 0, "Read %d bytes of memory into buffer %p from PID %d\n", info->size, (void*) info->buffer_addr, info->process_id);
		}
	}
	else if (info->code == CODE_WRITE_MEMORY)
	{
		PEPROCESS target_process = NULL;
		if (NT_SUCCESS(PsLookupProcessByProcessId((HANDLE)info->process_id, &target_process)))
		{
			memory::write_memory(target_process, (void*) info->buffer_addr, (void*)info->address, info->size);
		}
	}
}

