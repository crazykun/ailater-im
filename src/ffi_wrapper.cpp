/*
 * ailater-im - C++ wrapper for fcitx5 plugin
 *
 * This file provides the C++ interface that fcitx5 expects,
 * and forwards calls to the Rust implementation.
 */

#include <fcitx/addonfactory.h>
#include <fcitx/addonmanager.h>
#include <fcitx/instance.h>
#include <fcitx/inputmethodengine.h>
#include <fcitx/inputmethodentry.h>
#include <fcitx/inputpanel.h>
#include <fcitx/event.h>
#include <fcitx/candidatelist.h>
#include <fcitx-utils/i18n.h>
#include <fcitx-utils/log.h>

#include <string>
#include <memory>
#include <vector>

// External Rust functions
extern "C" {
    void* ailater_engine_create(void* instance);
    void ailater_engine_destroy(void* engine);
    int ailater_engine_handle_key(void* engine, void* ic, uint32_t keysym, uint32_t keycode, uint32_t state, bool is_release);
    void ailater_engine_reset(void* engine, void* ic);
    const char* ailater_engine_get_preedit(void* engine, void* ic);
    const char* ailater_engine_get_commit_text(void* engine, void* ic);
    const char** ailater_engine_get_candidates(void* engine, void* ic);
    size_t ailater_engine_get_candidate_count(void* engine, void* ic);
    size_t ailater_engine_get_current_page(void* engine, void* ic);
    void ailater_engine_free_string(char* s);
}

namespace fcitx {

// Custom CandidateWord that calls Rust engine when selected
class AilaterCandidateWord : public CandidateWord {
public:
    AilaterCandidateWord(const std::string& text, int index, void* engine, InputContext* ic)
        : CandidateWord(Text(text)), index_(index), engine_(engine), ic_(ic) {}

    void select(InputContext* inputContext) const override {
        // Simulate number key press to select this candidate
        // keysym for '1' is 0x31, so for index 0 we use 0x31, etc.
        uint32_t keysym = 0x30 + (index_ + 1);
        ailater_engine_handle_key(engine_, ic_, keysym, 0, 0, false);
        // Force UI update
        inputContext->updateUserInterface(UserInterfaceComponent::InputPanel);
    }

private:
    int index_;
    void* engine_;
    InputContext* ic_;
};

class AilaterEngine : public InputMethodEngineV2 {
public:
    AilaterEngine(Instance* instance) : instance_(instance) {
        engine_ = ailater_engine_create(instance);
        FCITX_INFO() << "AilaterEngine created";
    }

    ~AilaterEngine() override {
        if (engine_) {
            ailater_engine_destroy(engine_);
        }
        FCITX_INFO() << "AilaterEngine destroyed";
    }

    void activate(const InputMethodEntry& entry, InputContextEvent& event) override {
        FCITX_UNUSED(entry);
        FCITX_UNUSED(event);
        FCITX_INFO() << "AilaterEngine activated";
    }

    void deactivate(const InputMethodEntry& entry, InputContextEvent& event) override {
        FCITX_UNUSED(entry);
        FCITX_UNUSED(event);
        FCITX_INFO() << "AilaterEngine deactivated";
    }

    void keyEvent(const InputMethodEntry& entry, KeyEvent& keyEvent) override {
        FCITX_UNUSED(entry);

        auto* ic = keyEvent.inputContext();
        Key key = keyEvent.key();

        if (keyEvent.isRelease()) {
            return;
        }

        uint32_t keysym = key.sym();
        uint32_t keycode = key.code();
        uint32_t state = 0;

        if (key.states().test(KeyState::Shift)) state |= (1 << 0);
        if (key.states().test(KeyState::Ctrl)) state |= (1 << 1);
        if (key.states().test(KeyState::Alt)) state |= (1 << 2);
        if (key.states().test(KeyState::Super)) state |= (1 << 3);

        int result = ailater_engine_handle_key(engine_, ic, keysym, keycode, state, keyEvent.isRelease());

        if (result == 2) {  // CONSUME
            keyEvent.filterAndAccept();
        } else if (result == 1) {  // FORWARD
            keyEvent.filter();
        }

        updateUI(ic);
    }

    void reset(const InputMethodEntry& entry, InputContextEvent& event) override {
        FCITX_UNUSED(entry);
        ailater_engine_reset(engine_, event.inputContext());
        updateUI(event.inputContext());
    }

    void updateUI(InputContext* ic) {
        // Check if there's text to commit
        const char* commitText = ailater_engine_get_commit_text(engine_, ic);
        if (commitText && *commitText) {
            ic->commitString(std::string(commitText));
            // After commit, clear input panel
            ic->inputPanel().setPreedit(Text());
            ic->inputPanel().setCandidateList(nullptr);
            ic->updatePreedit();
            ic->updateUserInterface(UserInterfaceComponent::InputPanel);
            return;
        }

        // Get preedit text from Rust
        const char* preedit = ailater_engine_get_preedit(engine_, ic);
        if (preedit && *preedit) {
            ic->inputPanel().setPreedit(Text(preedit));
        } else {
            ic->inputPanel().setPreedit(Text());
        }

        // Get candidates from Rust and populate candidate list
        const char** candidates = ailater_engine_get_candidates(engine_, ic);
        if (candidates && *candidates) {
            auto candidateList = std::make_unique<CommonCandidateList>();

            // Get page size from fcitx5 global config
            int pageSize = instance_->globalConfig().defaultPageSize();
            candidateList->setPageSize(pageSize);

            // Set labels to 1-pageSize (e.g., "1", "2", ..., "9" for page size 9)
            std::vector<std::string> labels;
            for (int i = 0; i < pageSize && i < 9; i++) {
                labels.push_back(std::to_string(i + 1));
            }
            candidateList->setLabels(labels);

            // Add all candidates (CommonCandidateList handles paging)
            for (int i = 0; candidates[i] != nullptr; i++) {
                auto candidateWord = std::make_unique<AilaterCandidateWord>(
                    candidates[i], i, engine_, ic
                );
                candidateList->append(std::move(candidateWord));
            }

            // Set cursor to first candidate of current page
            size_t currentPage = ailater_engine_get_current_page(engine_, ic);
            candidateList->setCursorIndex(currentPage * pageSize);

            ic->inputPanel().setCandidateList(std::move(candidateList));
        } else {
            ic->inputPanel().setCandidateList(nullptr);
        }

        ic->updatePreedit();
        ic->updateUserInterface(UserInterfaceComponent::InputPanel);
    }

private:
    Instance* instance_;
    void* engine_;
};

class AilaterEngineFactory : public AddonFactory {
public:
    AddonInstance* create(AddonManager* manager) override {
        return new AilaterEngine(manager->instance());
    }
};

}  // namespace fcitx

// Use FCITX_ADDON_FACTORY macro - this creates the fcitx_addon_factory_instance symbol
FCITX_ADDON_FACTORY(fcitx::AilaterEngineFactory);
