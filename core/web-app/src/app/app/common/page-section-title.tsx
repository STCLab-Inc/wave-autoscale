export function PageSectionTitle({ title }: { title: string }) {
  return (
    <div className="flex h-10 w-full items-center justify-start">
      <div className="text-md font-semibold text-wa-gray-700">{title}</div>
    </div>
  );
}
