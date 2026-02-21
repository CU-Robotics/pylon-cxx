#pragma once

#include "rust/cxx.h"
#include <pylon/PylonIncludes.h>
#include <cstdint>

namespace Pylon {

struct BufferFactoryPolicy;

class BufferFactoryShim : public IBufferFactory {
public:
  explicit BufferFactoryShim(rust::Box<BufferFactoryPolicy> policy);
  virtual ~BufferFactoryShim() override {}
  virtual void AllocateBuffer(std::size_t buffer_size, void** created_buffer, std::intptr_t& buffer_ctx) override;
  virtual void FreeBuffer(void* created_buffer, std::intptr_t buffer_ctx) override {}
  virtual void DestroyBufferFactory() override {}

private:
  rust::Box<BufferFactoryPolicy> policy_;
};

std::unique_ptr<BufferFactoryShim> create_buffer_factory(rust::Box<BufferFactoryPolicy> policy);
void instant_camera_set_buffer_factory(const std::unique_ptr<CInstantCamera>& camera, const std::unique_ptr<BufferFactoryShim>& shim);

}