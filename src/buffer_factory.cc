#include "buffer_factory.h"
#include "pylon-cxx/src/lib.rs.h"
#include <memory>
#include <cstdio>

namespace Pylon {

BufferFactoryShim::BufferFactoryShim(rust::Box<BufferFactoryPolicy> policy) : policy_(std::move(policy)) {}

void BufferFactoryShim::AllocateBuffer(std::size_t buffer_size, void** created_buffer, std::intptr_t& buffer_ctx) {
    *created_buffer = reinterpret_cast<void*>(policy_->allocate(buffer_size));
    buffer_ctx = (intptr_t) nullptr; // context is unused and we want to avoid type erasure
}

std::unique_ptr<BufferFactoryShim> create_buffer_factory(rust::Box<BufferFactoryPolicy> policy) {
  return std::make_unique<BufferFactoryShim>(std::move(policy));
}

void instant_camera_set_buffer_factory(const std::unique_ptr<CInstantCamera>& camera, const std::unique_ptr<BufferFactoryShim>& shim) {
    camera->SetBufferFactory(shim.get(), Cleanup_None);
}

}