#pragma once

#include "ntapi.hpp"

namespace request_types {
	enum kernel_request_type : UINT8 {
		ReadMemory = 1,
		WriteMemory,
		GetModule,
		GetPebBase,
	};

	struct kernel_request
	{
		kernel_request_type request_type;
		PVOID buf;
		NTSTATUS status;
	};

	struct read_memory
	{
		UINT32 pid;
		UINT64 address;
		UINT64 size;
		PVOID read_buffer;
	};

	struct write_memory
	{
		UINT32 pid;
		UINT64 address;
		UINT64 size;
		PVOID write_buffer;
	};

	struct get_module
	{
		UINT32 pid;
		BOOL   is_64_bit;
		LPCWSTR module_name;
		UINT64 module_base;
		UINT64 module_size;
	};

	struct get_peb_base
	{
		UINT32 pid;
		UINT64 peb_base;
	};
}
