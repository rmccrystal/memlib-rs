#include "dispatch.hpp"

void dispatch::handler(void* req_ptr)
{
	using namespace request_types;
	//-DbgPrintEx(0, 0, "Hook was called with info struct %p\n", req_ptr);
	auto req = (kernel_request*)req_ptr;

	switch (req->request_type)
	{
	case kernel_request_type::ReadMemory:
	{
		auto read_req = (read_memory*)req->buf;
		//-DbgPrintEx(0, 0, "Reading %ull bytes of memory from PID %ul\n", read_req->size, read_req->pid);
		PEPROCESS target_process = NULL;
		req->status = PsLookupProcessByProcessId((HANDLE)read_req->pid, &target_process);
		if (NT_SUCCESS(req->status))
		{
			req->status = memory::read_memory(target_process, (void*)read_req->address, (void*) read_req->read_buffer, read_req->size);
			//-DbgPrintEx(0, 0, "Read %d bytes of memory into buffer %p from PID %d\n", read_req->size, (void*) read_req->read_buffer, read_req->pid);
		}
		break;
	}
	case kernel_request_type::WriteMemory:
	{
		auto write_req = (write_memory*)req->buf;
		PEPROCESS target_process = NULL;
		req->status = PsLookupProcessByProcessId((HANDLE)write_req->pid, &target_process);
		if (NT_SUCCESS(req->status))
		{
			req->status = memory::write_memory(target_process, (void*)write_req->write_buffer, (void*)write_req->address, write_req->size);
		}
		break;
	}
	case kernel_request_type::GetModule:
	{
		auto module_req = (get_module*)req->buf;

		if (module_req->module_name == 0) {
			req->status = 1;
			return;
		}

		PEPROCESS target_process = NULL;
		req->status = PsLookupProcessByProcessId((HANDLE)module_req->pid, &target_process);
		if (NT_SUCCESS(req->status))
		{
			// Clone the module name
			auto len = wcslen(module_req->module_name) + 1;
			LPWSTR module_name = (LPWSTR) ExAllocatePoolWithTag(PagedPool, len, 'tag9');
			if (module_name == 0) {
				req->status = 99;
				ExFreePoolWithTag(module_name, 'tag9');
				return;
			}

			wcscpy(module_name, module_req->module_name);

			auto is_64_bit = module_req->is_64_bit;
				
			//-DbgPrintEx(0, 0, "Module_name: %S, len: %zu\n", module_name, len);
			KAPC_STATE apc;
			KeStackAttachProcess(target_process, &apc);
			memory::module_info info = memory::module_info{ 0 };
			if (is_64_bit) {
				info = memory::get_module_info_64(target_process, module_name);
			}
			else {
				info = memory::get_module_info_32(target_process, module_name);
			}
			KeUnstackDetachProcess(&apc);

			// Free the temp module_name
			ExFreePoolWithTag(module_name, 'tag9');

			// Update the request
			module_req->module_base = info.base_address;
			module_req->module_size = info.size;
		}
		break;
	}
	case kernel_request_type::GetPebBase:
	{
		auto peb_request = (get_peb_base*)req->buf;

		PEPROCESS target_process = NULL;
		req->status = PsLookupProcessByProcessId((HANDLE)peb_request->pid, &target_process);
		if (NT_SUCCESS(req->status))
		{
			peb_request->peb_base = (UINT64)PsGetProcessPeb(target_process);
		}
	}
	default:
		//-DbgPrintEx(0, 0, "Invalid request type %d", req->request_type);
		break;
	}

	/*
	if (info->code == CODE_CLIENT_REQUEST)
	{
		PEPROCESS target_process = NULL;
		if (NT_SUCCESS(PsLookupProcessByProcessId((HANDLE)info->process_id, &target_process)))
		{
		}
	}
	else if (info->code == CODE_READ_MEMORY)
	{
	}
	else if (info->code == CODE_WRITE_MEMORY)
	{
		PEPROCESS target_process = NULL;
		if (NT_SUCCESS(PsLookupProcessByProcessId((HANDLE)info->process_id, &target_process)))
		{
			memory::write_memory(target_process, (void*) info->buffer_addr, (void*)info->address, info->size);
		}
	}
	*/
}

