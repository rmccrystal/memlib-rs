#pragma once

#include "hook.hpp"

namespace memory
{
	struct module_info
	{
		UINT64 base_address;
		UINT64 size;
	};

	module_info get_module_info_32(PEPROCESS target_process, LPCWSTR module_name);
	module_info get_module_info_64(PEPROCESS target_process, LPCWSTR module_name);

	ULONG get_module_base(PEPROCESS process, LPWSTR module_name);

	NTSTATUS read_memory(PEPROCESS target_process, void* source, void* target, size_t size);

	NTSTATUS write_memory(PEPROCESS target_process, void* source, void* target, size_t size);
}