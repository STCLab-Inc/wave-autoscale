import Link from 'next/link';
import ContentHeader from '../content-header';
import ScalingComponentService from '@/services/scaling-component';
import { ScalingComponentDefinition } from '@/types/bindings/scaling-component-definition';

async function getScalingComponents() {
  const components = await ScalingComponentService.getScalingComponents();
  return components;
}

export default async function ScalingComponentsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const components = await getScalingComponents();
  console.log({ components: components });
  return (
    <div className="flex h-full w-full">
      <main className="relative flex flex-1 flex-col">
        <div>
          <ContentHeader
            title="Scaling Components"
            right={
              <div className="flex items-center">
                <Link
                  href="/app/scaling-components/new"
                  className="no-underline"
                >
                  <button className="btn-primary btn-sm btn">
                    Add Component
                  </button>
                </Link>
              </div>
            }
          />
          <div className="w-full overflow-x-auto">
            <table className="table-compact table w-full">
              {/* head */}
              <thead>
                <tr>
                  <th>
                    <label>
                      <input type="checkbox" className="checkbox" />
                    </label>
                  </th>
                  <th>Component Kind</th>
                  <th>ID</th>
                  <th>Metadata</th>
                  <th>Actions</th>
                </tr>
              </thead>
              <tbody>
                {components.map((component: ScalingComponentDefinition) => {
                  const metadata = Object.keys(component.metadata)
                    .sort()
                    .map((key) => (
                      <div key={key}>
                        <span className="font-bold">{key}</span>:{' '}
                        {(component.metadata as any)[key]}
                      </div>
                    ));
                  return (
                    <tr key={component.db_id}>
                      <th>
                        <label>
                          <input type="checkbox" className="checkbox" />
                        </label>
                      </th>
                      <td>{component.component_kind}</td>
                      <td>{component.id}</td>
                      <td>{metadata}</td>
                      <td>
                        <Link
                          href={`/app/scaling-components/${component.db_id}`}
                        >
                          <button className="btn-primary btn-xs btn">
                            details
                          </button>
                        </Link>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
              {/* foot */}
              <tfoot>
                <tr>
                  <th></th>
                  <th></th>
                  <th></th>
                  <th></th>
                  <th></th>
                </tr>
              </tfoot>
            </table>
          </div>
        </div>
        {children}
      </main>
    </div>
  );
}
