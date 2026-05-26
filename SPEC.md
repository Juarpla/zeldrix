# Local AI Workstation - Product Vision Specification

Excellent decision! Turning any company PC into an internet-independent Local AI Workstation is a powerful value proposition, especially for sectors with sensitive data (legal, finance, health, human resources).

For the application to be indispensable in daily corporate life, it must go beyond a simple AI chat. It must become the "operating system" for the company's office tasks.

Here are the key functions your software should perform to completely transform local workflows:

## 1. The Intelligent Corporate Document Editor

This is the star feature. It's not just about writing, but about automating company bureaucracy using smart templates.

*   **Advanced Template Generation:** The user uploads or selects a standard company template (a confidentiality agreement, a warning letter, a commercial quote, or a technical report) where the fixed variables are marked (e.g., `[Client_Name]`, `[Contract_Amount]`).
*   **AI-Assisted Forms:** The user only fills in a couple of free-text fields or a quick summary of what they want ("Software services contract for company X, for 6 months, monthly payment of $2000, delivery at the end of each month"). The AI processes that structured summary with efficient models like Gemma-4 and fills in and integrates the legal or commercial template with impeccable and formal wording.
*   **Inline AI Tools:** When selecting any text within the editor (Notion or Medium style), a floating menu appears to: Instantly translate, change to a formal/persuasive tone, correct spelling, or expand on a technical point.

## 2. The Corporate "Smart Search" (Local RAG)

Companies waste hours searching for information in their own historical files. Your app can act as a local brain that knows everything about the company without uploading anything to the cloud.

*   **Local Document Indexing:** The user drags an entire company folder (PDFs, Excels, Word, technical manuals). The application processes the files locally, extracts the text, and generates a knowledge base.
*   **Audit and Semantic Query:** An accounting or legal employee can ask: "What were the penalty conditions in the contract we made with supplier X last year?". The app searches the indexed local documents and drafts the exact answer, citing the original document.

## 3. Automation of Repetitive Office Tasks

One-click actions to crush daily administrative work:

*   **Executive Summary of Email Threads:** The user copies a long and chaotic thread of corporate emails, and the app cleanly generates three things: a summary of the situation, the agreements reached, and a list of pending tasks for each person.
*   **Structured Data Extraction (Internal Data Scraping):** Imagine the administration department receives 50 invoices or CVs in PDF. They drag them into the app, and the AI reads the PDFs locally and automatically extracts the data (Name, ID/RUC, Amount, Date), exporting them directly to a clean Excel table.

## 4. Operating System Integrated Assistant

For the PC to truly feel like an AI workstation, the assistant must be a keyboard shortcut away.

*   **Quick Access Bar (Spotlight/Raycast Style):** Pressing `Alt + Space` opens a minimalist bar anywhere in the operating system. From there, the user can dictate a quick command or request a translation without having to open the full application interface.
*   **Lightweight Code Copilot Mode:** Useful if the company has a technical area or data analysts. The 35B Qwen model (in its ultra-quantized version) is fantastic for generating complex Excel macros, Python automation scripts, or local SQL queries for the company's database.

## 5. Interactive Assistance

To provide a more seamless and contextual help, the assistant will have the following capabilities:

*   **Voice Dictation and Commands:** The user can speak directly to the application to dictate content, issue commands, or ask questions. This allows for a hands-free and more natural interaction.
*   **Screen Sharing for Contextual Help:** The user can choose to share their screen with the application. This will allow the AI to "see" what the user is doing and provide more accurate and relevant assistance, for example, by guiding them through a complex software or helping them fill out a form.

## 6. The Hardware Control Panel (IT Admin View)

Designed for the company's systems manager. They need to see that the software will not freeze the worker's computer.

*   **Dynamic Layer Allocation (VRAM/RAM):** A very intuitive visual interface that allows configuring how much of the PC's power is assigned to the AI. If the secretary's PC has an entry-level video card, the system configures `llama.cpp` to use partial GPU acceleration (`-ngl`). If it is a basic office PC, it is optimized for low-priority background CPU threads so that the user can continue using Word or Chrome without slowness.

## Workstation Workflow Architecture

1.  **Ingestion and Selection:**
    *   **Phase 1.** The user opens the application, selects the task ("Generate Meeting Minutes"), and chooses the company's pre-configured corporate template.
2.  **Local Contextualization:**
    *   **Phase 2.** The raw context is added (e.g., the locally transcribed audio or the quick block notes from the meeting).
3.  **Inference in Llama.cpp:**
    *   **Phase 3.** The Tauri command awakens the local model (Gemma or Qwen), passing the optimized hardware parameters of that specific PC.
4.  **Aesthetic Rendering:**
    *   **Phase 4.** The processed text is injected in real-time (streaming) directly into the web interface of the document editor, ready to be exported to PDF or Word with an impeccable design.