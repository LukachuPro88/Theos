# Contributing to Theos

Thanks for your interest in contributing! This guide covers both the workflow for getting changes merged and the code standards we try to hold ourselves to. The rules here are guidelines — follow them unless you have a good reason not to, and be ready to explain that reason in your PR.

---

## Workflow

### 1. Fork & Branch

Fork the repository and create a branch off `main`. Name your branch after what you're doing:

```
feat/vga-driver
fix/boot-hang-real-mode
refactor/gdt-setup
```

Prefixes: `feat/`, `fix/`, `refactor/`, `docs/`, `chore/`.

### 2. Make Your Changes

Keep changes focused. A PR that fixes a bug and rewrites an unrelated module is harder to review and easier to break. If you spot something unrelated that needs fixing, open a separate issue or PR.

### 3. Test Before You Push

Run the OS in QEMU and make sure it still boots cleanly:

```bash
make run
```

If your change touches the bootloader or mode transitions, verify the full boot sequence manually.

### 4. Open a Pull Request

- Write a clear title: `Fix stack alignment in Protected Mode transition`
- In the description, explain *what* changed and *why* — not just what the diff shows
- Link any related issues with `Closes #123` or `Relates to #456`

### 5. Review

Be receptive to feedback. If a reviewer asks for changes, discuss it — don't just push back. If you disagree, explain your reasoning.

---

## Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) format:

```
<type>(<scope>): <short summary>

<optional body>
```

**Types:** `feat`, `fix`, `refactor`, `docs`, `chore`, `test`

**Rules:**
- Use the imperative mood in the summary: `add`, not `added` or `adds`
- Keep the summary under 72 characters
- Use the body to explain *why*, not *what* — the diff shows what

**Examples:**

```
feat(boot): add A20 line enable before protected mode entry

fix(kernel): correct stack pointer alignment for 16-byte boundary

docs(readme): clarify long mode transition steps
```

---

## Naming Conventions

### Rust

Follow standard Rust conventions. When in doubt, `rustfmt` is the authority.

| Thing | Convention | Example |
|---|---|---|
| Types / Traits | `UpperCamelCase` | `GlobalDescriptorTable` |
| Functions / Methods | `snake_case` | `enable_protected_mode()` |
| Constants | `SCREAMING_SNAKE_CASE` | `GDT_ENTRY_COUNT` |
| Modules | `snake_case` | `mod memory_map` |
| Type parameters | Single uppercase letter or short `UpperCamelCase` | `T`, `Err` |

### Assembly

- Labels: `snake_case` with a dot prefix for local labels (`.loop`, `.done`)
- Constants defined with `equ`: `SCREAMING_SNAKE_CASE`
- Prefix internal-only labels with an underscore: `_setup_stack`

---

## Documentation & Comments

### When to comment

Comment *why*, not *what*. If the code clearly shows what's happening, a comment restating it adds noise. If there's a non-obvious reason for a choice — hardware quirk, spec reference, workaround — that's worth a comment.

```rust
// Bad: restates the code
gdt.set_base(0); // set base to 0

// Good: explains the why
// Flat memory model — base and limit cover the full 4GB address space
// as required before entering long mode.
gdt.set_base(0);
```

### Doc comments

Public functions and types should have a `///` doc comment. Include:

- A one-line summary
- Parameter and return notes if non-obvious
- Panics or safety requirements for `unsafe` functions

```rust
/// Loads the Global Descriptor Table and reloads all segment registers.
///
/// # Safety
/// The caller must ensure `gdt` remains valid for the lifetime of the program.
pub unsafe fn load(gdt: &GlobalDescriptorTable) { ... }
```

### Assembly comments

Comment every non-trivial instruction block. Reference the Intel manual or OSDev Wiki where relevant.

```nasm
; Enable the A20 line via the keyboard controller (method 2)
; See: https://wiki.osdev.org/A20_Line
mov al, 0xDD
out 0x64, al
```

---

## Error Handling

Theos runs without a standard library, so there's no `std::error::Error` or panicking allocator to fall back on. Keep this in mind:

- **Prefer `Result<T, E>`** for fallible operations. Define narrow, descriptive error enums rather than using a catch-all.
- **Avoid `unwrap()` and `expect()`** in kernel code. If something can fail, handle it explicitly. `expect()` is acceptable in early boot setup where a failure is truly unrecoverable and you want a clear message.
- **No silent failures.** If an error is ignored intentionally, mark it with a comment explaining why.
- **Panics are last resort.** A kernel panic should mean something went badly wrong, not that an edge case wasn't handled.

```rust
// Prefer this:
match load_segment(selector) {
    Ok(seg) => seg,
    Err(e) => return Err(KernelError::SegmentLoad(e)),
}

// Over this:
let seg = load_segment(selector).unwrap();
```

---

## Code Style

Run `rustfmt` before committing Rust code — this is non-negotiable:

```bash
cargo fmt
```

Run `clippy` and address any warnings. If you deliberately ignore a lint, add `#[allow(...)]` with a comment explaining why:

```rust
#[allow(clippy::cast_possible_truncation)]
// Truncation is intentional: we only need the low 16 bits of the address here.
let low = addr as u16;
```

---

## Questions

Not sure about something? Open an issue and ask before writing a lot of code. It's much easier to discuss an approach before it's implemented than after.