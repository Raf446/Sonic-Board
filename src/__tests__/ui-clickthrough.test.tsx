// @vitest-environment jsdom
import { describe, expect, it } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import App from "../App";

describe("SonicBoard Full UI Click-Through & Theme Verification (R-35)", () => {
  it(
    "clicks through every interactive control across Dark and Daylight themes with zero errors",
    async () => {
      const { container, unmount } = render(<App />);

      await waitFor(() => {
        expect(screen.getByText("Bass Drop Impact")).toBeTruthy();
      });

      // 1. Theme Toggle: Dark Rack -> Daylight -> Dark Rack
      expect(document.documentElement.classList.contains("dark")).toBe(true);
      fireEvent.click(screen.getByRole("button", { name: /Ganti ke mode terang/i }));
      expect(document.documentElement.classList.contains("light")).toBe(true);
      fireEvent.click(screen.getByRole("button", { name: /Ganti ke mode gelap/i }));
      expect(document.documentElement.classList.contains("dark")).toBe(true);

      // 2. Custom Window Controls: Minimize, Maximize, Close
      fireEvent.click(screen.getByRole("button", { name: /Minimize jendela/i }));
      expect(screen.getByText(/Jendela diminimalkan/i)).toBeTruthy();
      fireEvent.click(screen.getByRole("button", { name: /Maximize jendela/i }));
      expect(screen.getByText(/Ukuran jendela diubah/i)).toBeTruthy();
      fireEvent.click(screen.getByRole("button", { name: /Tutup aplikasi/i }));

      // 3. Search Input & Reset button
      const searchInput = screen.getByRole("searchbox", {
        name: /Cari suara atau hotkey/i,
      });
      fireEvent.change(searchInput, { target: { value: "NonExistentSoundXYZ" } });
      expect(
        screen.getByText(/Tidak ada sampel suara di tampilan ini/i)
      ).toBeTruthy();
      fireEvent.click(screen.getByRole("button", { name: /Bersihkan pencarian/i }));
      expect(screen.getByText("Bass Drop Impact")).toBeTruthy();

      // 4. Category Filter Pills: Gaming, Favorit, Semua
      fireEvent.click(screen.getByRole("button", { name: /^Gaming/i }));
      expect(screen.getByText("Tactical Clutch Alert")).toBeTruthy();
      fireEvent.click(screen.getByRole("button", { name: /^Favorit/i }));
      expect(screen.getByText("Synth Chime Spark")).toBeTruthy();
      fireEvent.click(screen.getByRole("button", { name: /^Semua/i }));

      // 5. "+ Kategori Baru" Modal: open, submit new category, verify active
      fireEvent.click(screen.getByRole("button", { name: /\+ Kategori Baru/i }));
      const catInput = screen.getByLabelText(/Nama Kategori/i);
      fireEvent.change(catInput, { target: { value: "Turnamen" } });
      fireEvent.submit(catInput.closest("form")!);
      await waitFor(() => {
        expect(screen.getByText(/Kategori: Turnamen/i)).toBeTruthy();
      });
      fireEvent.click(screen.getByRole("button", { name: /Reset Filter/i }));

      // 6. Sound Pad Card Controls: Play, Mute/Unmute, Volume Slider, Favorite Star
      fireEvent.click(
        screen.getByRole("button", { name: /Trigger sampel Bass Drop Impact/i })
      );
      fireEvent.click(screen.getByRole("button", { name: /Mute Bass Drop Impact/i }));
      expect(
        screen.getByRole("button", { name: /Unmute Bass Drop Impact/i })
      ).toBeTruthy();

      fireEvent.change(
        screen.getByRole("slider", { name: /Volume Bass Drop Impact/i }),
        { target: { value: "75" } }
      );
      fireEvent.click(
        screen.getByRole("button", {
          name: /Hapus Bass Drop Impact dari Favorit/i,
        })
      );

      // 7. Edit Sound Modal via Card Edit Button
      fireEvent.click(screen.getByRole("button", { name: /^Edit Bass Drop Impact$/i }));
      const nameInput = screen.getByLabelText(/Nama Suara/i);
      fireEvent.change(nameInput, { target: { value: "Bass Drop Impact Pro" } });
      fireEvent.submit(nameInput.closest("form")!);
      await waitFor(() => {
        expect(screen.getByText("Bass Drop Impact Pro")).toBeTruthy();
      });

      // 8. Dashed Drop-Zone Card ("+ Tambah Suara Baru"): open modal, add new sound
      fireEvent.click(
        screen.getByRole("button", {
          name: /Tambah Suara Baru \(Import MP3\/WAV, setting hotkey\)/i,
        })
      );
      const addNameInput = screen.getByLabelText(/Nama Suara/i);
      fireEvent.change(addNameInput, {
        target: { value: "Custom Raid Horn" },
      });
      fireEvent.change(screen.getByLabelText(/Global Hotkey/i), {
        target: { value: "Ctrl + 7" },
      });
      fireEvent.submit(addNameInput.closest("form")!);
      await waitFor(() => {
        expect(screen.getByText("Custom Raid Horn")).toBeTruthy();
      });

      // 9. Right Sidebar: Input & Output Routing Selectors & Refresh Button
      const micSelect = screen.getByLabelText(/Physical Mic/i) as HTMLSelectElement;
      fireEvent.change(micSelect, { target: { value: micSelect.options[0].value } });
      const virtSelect = screen.getByLabelText(/Virtual Mic/i) as HTMLSelectElement;
      fireEvent.change(virtSelect, { target: { value: virtSelect.options[0].value } });
      const monSelect = screen.getByLabelText(
        /Monitor \(Headphone Output\)/i
      ) as HTMLSelectElement;
      fireEvent.change(monSelect, { target: { value: monSelect.options[0].value } });
      fireEvent.click(
        screen.getByRole("button", { name: /Segarkan daftar perangkat audio/i })
      );

      // 10. Virtual Mixer Vertical Sliders (Mic Vol, Soundboard, Monitor)
      fireEvent.change(screen.getByRole("slider", { name: /Volume Mic Vol/i }), {
        target: { value: "80" },
      });
      fireEvent.change(screen.getByRole("slider", { name: /Volume Soundboard/i }), {
        target: { value: "95" },
      });
      fireEvent.change(
        screen.getByRole("slider", { name: /Volume Monitor \(Headphone\)/i }),
        { target: { value: "70" } }
      );

      // 11. Voice FX (DSP Rotary Knobs: Pitch, Reverb, Distortion + Keyboard ArrowUp + Reset)
      fireEvent.keyDown(
        screen.getByRole("slider", { name: /Kontrol rotari DSP Pitch/i }),
        { key: "ArrowUp" }
      );
      fireEvent.keyDown(
        screen.getByRole("slider", { name: /Kontrol rotari DSP Reverb/i }),
        { key: "ArrowUp" }
      );
      fireEvent.keyDown(
        screen.getByRole("slider", { name: /Kontrol rotari DSP Distortion/i }),
        { key: "ArrowUp" }
      );
      fireEvent.click(
        screen.getByRole("button", { name: /Reset efek suara DSP ke nol/i })
      );

      // 12. Emergency Action: PANIC MUTE & Resume Audio Routing
      fireEvent.click(screen.getByRole("button", { name: /PANIC MUTE/i }));
      await waitFor(() => {
        expect(screen.getByText("MUTED")).toBeTruthy();
      });
      fireEvent.click(
        screen.getByRole("button", {
          name: /AKTIFKAN KEMBALI ROUTING AUDIO/i,
        })
      );
      await waitFor(() => {
        expect(screen.getByText("CONNECTED")).toBeTruthy();
      });

      expect(container.querySelectorAll("article").length).toBeGreaterThanOrEqual(8);
      unmount();
      cleanup();
    },
    15000
  );
});
