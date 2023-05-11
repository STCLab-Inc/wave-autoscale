import PlanningSidebar from './sidebar';

export default function PlanningLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="flex h-full w-full">
      <PlanningSidebar />
      <main className="flex flex-1 flex-col">{children}</main>
    </div>
  );
}
