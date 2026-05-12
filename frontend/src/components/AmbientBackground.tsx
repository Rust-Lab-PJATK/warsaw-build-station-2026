export function AmbientBackground() {
  return (
    <div
      aria-hidden="true"
      className="pointer-events-none fixed inset-0 -z-10 overflow-hidden"
    >
      {/* Blob 1 — teal, top-left */}
      <div className="absolute -left-[15%] -top-[10%] h-[640px] w-[640px] rounded-full bg-[#44bcc3]/[0.08] blur-[120px] animate-blob" />

      {/* Blob 2 — deep teal, top-right, delayed 2 s */}
      <div className="absolute -top-[5%] right-[5%] h-[540px] w-[540px] rounded-full bg-[#0c1e2a]/[0.9] blur-[120px] animate-blob animation-delay-2000" />

      {/* Blob 3 — teal accent, bottom-center, delayed 4 s */}
      <div className="absolute bottom-[5%] left-[30%] h-[480px] w-[480px] rounded-full bg-[#44bcc3]/[0.05] blur-[120px] animate-blob animation-delay-4000" />

      {/* Film-grain noise — mix-blend-screen lifts it into the light */}
      <div className="noise absolute inset-0 opacity-[0.035] mix-blend-screen" />
    </div>
  );
}
