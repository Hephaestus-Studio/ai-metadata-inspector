import React from 'react'
import { ActionIcon, Badge, Button, Group } from '@mantine/core'
import { IconDownload, IconPlus, IconTrash, IconX } from '@tabler/icons-react'
import clsx from 'clsx'
import type { ImageMetadataReport } from '@/types/metadata'
import classes from './BatchQueue.module.css'

export interface BatchItem {
  id: string
  file: File
  previewUrl: string
  report?: ImageMetadataReport
  cleanedBytes?: Uint8Array
}

interface BatchQueueProps {
  items: BatchItem[]
  selectedIndex: number
  onSelectIndex: (index: number) => void
  onRemoveItem: (index: number) => void
  onClearAll: () => void
  onAddMore: () => void
  onBatchCleanAndDownloadZip: () => void
  cleaningBatch?: boolean
}

export const BatchQueue: React.FC<BatchQueueProps> = ({
  items,
  selectedIndex,
  onSelectIndex,
  onRemoveItem,
  onClearAll,
  onAddMore,
  onBatchCleanAndDownloadZip,
  cleaningBatch = false,
}) => {
  if (items.length <= 1) return null

  const totalSizeMb = (
    items.reduce((acc, it) => acc + it.file.size, 0) /
    (1024 * 1024)
  ).toFixed(1)

  return (
    <div className={classes.batchBar}>
      {/* Scrollable list of image thumbnail chips */}
      <div className={classes.chipsScrollArea}>
        {items.map((item, idx) => {
          const isActive = idx === selectedIndex
          return (
            <div
              key={item.id}
              className={clsx(
                classes.itemCard,
                isActive && classes.itemCardActive,
              )}
              onClick={() => onSelectIndex(idx)}
            >
              <div className={classes.thumbnailWrapper}>
                <img
                  src={item.previewUrl}
                  alt={item.file.name}
                  className={classes.thumbnail}
                />
              </div>
              <span className={classes.fileName}>{item.file.name}</span>
              {item.report && (
                <Badge
                  size="xs"
                  variant="filled"
                  color={
                    item.report.risk_report.level === 'safe'
                      ? 'teal'
                      : item.report.risk_report.level === 'medium'
                        ? 'orange'
                        : 'red'
                  }
                >
                  {item.report.risk_report.score}
                </Badge>
              )}
              <ActionIcon
                size="xs"
                variant="subtle"
                color="gray"
                onClick={(e) => {
                  e.stopPropagation()
                  onRemoveItem(idx)
                }}
              >
                <IconX size={12} />
              </ActionIcon>
            </div>
          )
        })}

        <Button
          size="xs"
          variant="subtle"
          color="indigo"
          leftSection={<IconPlus size={14} />}
          onClick={onAddMore}
          className={classes.addMoreBtn}
        >
          Add Image
        </Button>
      </div>

      {/* Right Batch Actions */}
      <Group gap={8} wrap="nowrap" style={{ flexShrink: 0 }}>
        <Button
          size="xs"
          variant="filled"
          color="teal"
          loading={cleaningBatch}
          leftSection={<IconDownload size={14} />}
          onClick={onBatchCleanAndDownloadZip}
          className={classes.batchCleanBtn}
        >
          Clean All ({items.length} files • {totalSizeMb} MB)
        </Button>

        <ActionIcon
          size="sm"
          color="red"
          variant="subtle"
          onClick={onClearAll}
          title="Clear Queue"
        >
          <IconTrash size={16} />
        </ActionIcon>
      </Group>
    </div>
  )
}
