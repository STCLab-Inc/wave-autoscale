'use client';

import React, { useEffect, useState } from 'react';
import Link from 'next/link';

import ContentHeader from '../content-header';
import ScalingComponentService from '@/services/scaling-component';
import { renderKeyValuePairsWithJson } from '../keyvalue-renderer';
import { ScalingComponentDefinition } from '@/types/bindings/scaling-component-definition';

async function getScalingComponents() {
  const components = await ScalingComponentService.getScalingComponents();
  return components;
}

interface ScalingComponentDefinitionEx extends ScalingComponentDefinition {
  isChecked: boolean;
}

export default function ScalingComponentsPage() {
  const [components, setComponents] = useState<ScalingComponentDefinitionEx[]>(
    []
  );
  const [checkAllFlag, setCheckAllFlag] = useState(false);

  useEffect(() => {
    const fetchComponents = async () => {
      try {
        const components = await getScalingComponents();
        setComponents(components);
      } catch (error) {
        console.error({ error });
      }
    };
    fetchComponents();
  }, []);

  const handleCheckChange = (isChecked: boolean) => {
    setCheckAllFlag(isChecked);
    const updatedComponents = components.map((item) => ({
      ...item,
      isChecked,
    }));
    setComponents(updatedComponents);
  };

  const ITEMS_PER_PAGE_OPTIONS = [10, 50, 100, 200, 500];

  const [itemsPerPage, setItemsPerPage] = useState(ITEMS_PER_PAGE_OPTIONS[0]);
  const [currentPage, setCurrentPage] = useState(1);

  const totalPageCount = Math.ceil(components.length / itemsPerPage);

  const startIndex = (currentPage - 1) * itemsPerPage;
  const endIndex = startIndex + itemsPerPage;
  const visibleComponents = components.slice(startIndex, endIndex);

  const handleItemsPerPageChange = (
    event: React.ChangeEvent<HTMLSelectElement>
  ) => {
    const newItemsPerPage = parseInt(event.target.value, 10);
    setItemsPerPage(newItemsPerPage);
    setCurrentPage(1);
  };

  const handlePageChange = (newPage: number) => {
    setCurrentPage(newPage);
  };

  return (
    <main className="flex h-full w-full flex-col">
      <div>
        <ContentHeader
          title="Scaling Components"
          right={
            <div className="flex items-center">
              <Link href="/app/scaling-components/new">
                <button className="flex h-8 items-center justify-center rounded-md border border-blue-400 bg-blue-400  pl-5 pr-5 text-sm text-gray-50">
                  ADD COMPONENT
                </button>
              </Link>
            </div>
          }
        />
        <div className="flex w-full flex-col">
          <div className="flex items-center justify-end px-8 py-4">
            <div className="mr-2 flex w-16 items-center">
              <label className="select-group-sm">
                <select
                  value={itemsPerPage}
                  onChange={handleItemsPerPageChange}
                  className="focus:outline-noneselect select-sm max-w-[130px] cursor-pointer rounded-md border border-gray-200 px-2"
                >
                  {ITEMS_PER_PAGE_OPTIONS.map((option) => (
                    <option key={option} value={option}>
                      {option}
                    </option>
                  ))}
                </select>
              </label>
            </div>

            <div className="mx-2 flex w-16 items-center justify-center">
              <span className="px-2 text-center text-sm">
                {currentPage} / {totalPageCount}
              </span>
            </div>

            <div className="ml-2 flex h-8 items-center">
              <button
                className={
                  currentPage === 1
                    ? 'ml-1 mr-1 flex h-8 cursor-not-allowed items-center justify-center rounded-md border border-gray-400 bg-gray-400 pl-5 pr-5 text-sm text-gray-50'
                    : 'ml-1 mr-1 flex h-8 cursor-pointer items-center justify-center rounded-md border border-red-400 bg-red-400 pl-5 pr-5 text-sm text-gray-50'
                }
                onClick={() => handlePageChange(currentPage - 1)}
                disabled={currentPage === 1}
              >
                PREVIOUS
              </button>
              <button
                className={
                  currentPage &&
                  totalPageCount &&
                  currentPage !== totalPageCount
                    ? 'ml-1 mr-1 flex h-8 cursor-pointer items-center justify-center rounded-md border border-blue-400 bg-blue-400 pl-5 pr-5 text-sm text-gray-50'
                    : 'ml-1 mr-1 flex h-8 cursor-not-allowed items-center justify-center rounded-md border border-gray-400 bg-gray-400 pl-5 pr-5 text-sm text-gray-50'
                }
                onClick={() => handlePageChange(currentPage + 1)}
                disabled={currentPage === totalPageCount}
              >
                NEXT
              </button>
            </div>
          </div>

          <table className="flex w-full flex-col">
            <thead className="text-md flex h-12 w-full items-center justify-between border-b border-t bg-gray-75 py-0 font-bold text-gray-800">
              <tr className="flex h-full w-full px-8">
                <th className="mr-4 flex h-full flex-1 items-center">
                  <label className="flex h-full items-center">
                    <input
                      type="checkbox"
                      className="checkbox"
                      checked={checkAllFlag}
                      onChange={(event) =>
                        handleCheckChange(event.target.checked)
                      }
                    />
                  </label>
                </th>
                <th className="mx-4 flex h-full w-full flex-8 items-center">
                  <span className="flex items-center break-words">
                    Scaling Component Kind
                  </span>
                </th>
                <th className="mx-4 flex h-full w-full flex-8 items-center">
                  <span className="flex items-center break-words">
                    Scaling Component ID
                  </span>
                </th>
                <th className="mx-4 flex h-full w-full flex-8 items-center">
                  <span className="flex items-center break-words">
                    Metadata Values
                  </span>
                </th>
                <th className="mx-4 flex h-full w-full flex-2 items-center">
                  <span className="flex items-center break-words">Actions</span>
                </th>
              </tr>
            </thead>
            <tbody className="text-md min-h-12 flex w-full flex-col items-center justify-between py-0 text-gray-800">
              {visibleComponents.map(
                (componentsItem: ScalingComponentDefinitionEx) => (
                  <tr
                    key={componentsItem.db_id}
                    className="flex w-full border-b px-8 py-4"
                  >
                    <td className="mr-4 flex h-full flex-1 items-start">
                      <label className="flex items-center">
                        <input
                          type="checkbox"
                          className="checkbox"
                          checked={componentsItem.isChecked}
                          onChange={(event) =>
                            handleCheckChange(event.target.checked)
                          }
                        />
                      </label>
                    </td>
                    <td className="mx-4 flex h-full w-full flex-8 items-start">
                      <div className="flex items-center break-all">
                        {componentsItem.component_kind}
                      </div>
                    </td>
                    <td className="mx-4 flex h-full w-full flex-8 items-start">
                      <div className="flex items-center break-all">
                        {componentsItem.id}
                      </div>
                    </td>
                    <td className="mx-4 flex h-full w-full flex-8 items-start">
                      <div className="flex flex-col items-center">
                        {renderKeyValuePairsWithJson(
                          JSON.stringify(componentsItem.metadata),
                          false
                        )}
                      </div>
                    </td>
                    <td className="mx-4 flex h-full w-full flex-2 items-start">
                      <div className="flex items-center">
                        <Link
                          href={`/app/scaling-components/${componentsItem.db_id}`}
                        >
                          <button className="badge-success badge bg-[#074EAB] px-2 py-3 text-white">
                            Details
                          </button>
                        </Link>
                      </div>
                    </td>
                  </tr>
                )
              )}
            </tbody>
          </table>
        </div>
      </div>
    </main>
  );
}
