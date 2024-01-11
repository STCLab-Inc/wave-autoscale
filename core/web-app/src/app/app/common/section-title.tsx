export function SectionTitle({ title }: { title: string }) {
  return (
    <div className="flex h-10 w-full items-center justify-start">
      <div className="text-lg font-bold">{title}</div>
    </div>
  );
}
