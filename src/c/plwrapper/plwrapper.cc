#include "plwrapper.h"

#include <cstring>
#include <format>
#include <fstream>
#include <memory>
#include <optional>
#include <pl/pattern_language.hpp>
#include <pl/patterns/pattern.hpp>
#include <pl/patterns/pattern_array_static.hpp>
#include <string>

namespace {

struct FieldRecord {
    std::string path;
    std::string typeName;
    std::string value;
    uint64_t offset;
    uint64_t size;
};

std::optional<std::vector<uint8_t>> readFile(const std::string &path) {
    std::ifstream file(path, std::ios::binary | std::ios::ate);
    if (!file.is_open()) {
        return std::nullopt;
    }

    auto size = file.tellg();
    if (size < 0) {
        return std::nullopt;
    }
    file.seekg(0, std::ios::beg);

    std::vector<uint8_t> data(static_cast<size_t>(size));
    if (size > 0 && !file.read(reinterpret_cast<char *>(data.data()), size)) {
        return std::nullopt;
    }

    return data;
}

std::string joinPath(const std::string &parent, const std::string &child) {
    if (parent.empty()) {
        return child;
    }
    if (!child.empty() && child.front() == '[') {
        return parent + child;
    }
    return parent + "." + child;
}

} // namespace

struct pl_plwrapper {
    std::unique_ptr<pl::PatternLanguage> patternLanguage;
    std::vector<uint8_t> fileData;
    std::vector<std::string> includePaths;
    std::string lastError;
    std::vector<FieldRecord> records;
    pl_result currentView{};

    void collect(const std::shared_ptr<pl::ptrn::Pattern> &pattern,
                 const std::string &parentPath) {
        using pl::ptrn::Visibility;

        auto visibility = pattern->getVisibility();
        if (visibility == Visibility::Hidden ||
            visibility == Visibility::TreeHidden) {
            return;
        }

        auto path = joinPath(parentPath, pattern->getVariableName());

        bool isStaticArray = dynamic_cast<pl::ptrn::PatternArrayStatic *>(
                                 pattern.get()) != nullptr;
        if (!isStaticArray) {
            if (auto *iterable =
                    dynamic_cast<pl::ptrn::IIterable *>(pattern.get())) {
                for (const auto &entry : iterable->getEntries())
                    this->collect(entry, path);
                return;
            }
        }

        this->records.emplace_back(FieldRecord{
            .path = path,
            .typeName = pattern->getTypeName(),
            .value = pattern->getFormattedValue(),
            .offset = pattern->getOffset(),
            .size = pattern->getSize(),
        });
    }
};

pl_plwrapper *pl_create(void) { return new pl_plwrapper(); }

void pl_destroy(pl_plwrapper *rt) { delete rt; }

void pl_add_include_path(pl_plwrapper *rt, const char *path) {
    if (rt == nullptr || path == nullptr) {
        return;
    }
    rt->includePaths.emplace_back(path);
}

int pl_execute_file(pl_plwrapper *rt, const char *pattern_path,
                    const char *data_path) {
    if (rt == nullptr || pattern_path == nullptr || data_path == nullptr) {
        return -1;
    }

    rt->lastError.clear();
    rt->records.clear();

    auto data = readFile(data_path);
    if (!data) {
        rt->lastError = std::string("failed to read data file: ") + data_path;
        return -1;
    }
    rt->fileData = std::move(*data);

    auto patternSource = readFile(pattern_path);
    if (!patternSource) {
        rt->lastError =
            std::string("failed to read pattern file: ") + pattern_path;
        return -1;
    }

    rt->patternLanguage = std::make_unique<pl::PatternLanguage>();
    auto &lang = *rt->patternLanguage;

    if (!rt->includePaths.empty()) {
        std::vector<std::fs::path> paths;
        paths.reserve(rt->includePaths.size());
        for (const auto &path : rt->includePaths) {
            paths.emplace_back(path);
        }
        lang.setIncludePaths(paths);
    }

    lang.setDangerousFunctionCallHandler([] { return false; });

    auto &fileData = rt->fileData;
    lang.setDataSource(
        0, fileData.size(),
        [&fileData](pl::u64 address, pl::u8 *buffer, size_t size) {
            if (address + size > fileData.size()) {
                std::memset(buffer, 0, size);
                return;
            }
            std::memcpy(buffer, fileData.data() + address, size);
        });

    std::string source(reinterpret_cast<const char *>(patternSource->data()),
                       patternSource->size());
    int result = lang.executeString(source, pattern_path);

    if (result != 0) {
        if (auto &evalError = lang.getEvalError(); evalError.has_value()) {
            rt->lastError = std::format("{}:{} {}", evalError->line,
                                        evalError->column, evalError->message);
            if (evalError->cursorAddress.has_value() &&
                *evalError->cursorAddress < rt->fileData.size()) {
                rt->lastError += std::format(" (cursor at offset 0x{:X})",
                                             *evalError->cursorAddress);
            }
        } else if (const auto &compileErrors = lang.getCompileErrors();
                   !compileErrors.empty()) {
            rt->lastError = compileErrors.front().format();
        } else {
            rt->lastError =
                "An unknown error occurred during pattern execution.";
        }
    }

    for (const auto &pattern : lang.getPatterns()) {
        rt->collect(pattern, "");
    }

    return result;
}

const char *pl_last_error(pl_plwrapper *rt) {
    if (rt == nullptr) {
        return "";
    }

    return rt->lastError.c_str();
}

size_t pl_field_count(pl_plwrapper *rt) {
    if (rt == nullptr) {
        return 0;
    }
    return rt->records.size();
}

const pl_result *pl_field_at(pl_plwrapper *rt, size_t index) {
    if (rt == nullptr || index >= rt->records.size()) {
        return nullptr;
    }

    const auto &record = rt->records[index];
    rt->currentView.path = record.path.c_str();
    rt->currentView.type_name = record.typeName.c_str();
    rt->currentView.value = record.value.c_str();
    rt->currentView.offset = record.offset;
    rt->currentView.size = record.size;
    return &rt->currentView;
}
