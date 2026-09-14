import React from 'react'
import {
  ActionIcon,
  Badge,
  CopyButton,
  Group,
  Stack,
  Text,
  Tooltip,
} from '@mantine/core'
import { IconCheck, IconCopy, IconFileCode } from '@tabler/icons-react'
import type { ImageMetadataReport, RawChunkInfo } from '@/types/metadata'
import { TabEmptyState } from './TabEmptyState'
import classes from './RawChunksTab.module.css'

interface RawChunksTabProps {
  chunks: RawChunkInfo[]
  rawChunksFound: string[]
  report?: ImageMetadataReport
  onNavigateTab?: (tab: string) => void
}

export const RawChunksTab: React.FC<RawChunksTabProps> = ({
  chunks,
  rawChunksFound,
  report,
  onNavigateTab,
}) => {
  if (chunks.length === 0 && rawChunksFound.length === 0) {
    return (
      <TabEmptyState
        icon={<IconFileCode size={28} stroke={1.5} />}
        title="No Raw Chunks or Text Markers"
        description="No custom container chunks (PNG tEXt/iTXt, JPEG markers, WebP user chunks) were detected in this image."
        currentTab="raw"
        report={report}
        onNavigateTab={onNavigateTab}
      />
    )
  }
  return (
    <Stack gap="md">
      {/* Chunk Types Summary */}
      {rawChunksFound.length > 0 && (
        <div>
          <Text size="xs" fw={700} c="dimmed" tt="uppercase" mb={6}>
            Container Chunk Types / Markers Detected ({rawChunksFound.length}):
          </Text>
          <Group gap="xs">
            {rawChunksFound.map((ct, idx) => (
              <Badge key={idx} variant="outline" color="indigo" size="sm">
                {ct}
              </Badge>
            ))}
          </Group>
        </div>
      )}

      {/* Raw Text Chunks Payload List */}
      <div>
        <Text size="xs" fw={700} c="dimmed" tt="uppercase" mb={4}>
          Decoded Key-Value Text Chunks ({chunks.length}):
        </Text>

        {chunks.length === 0 ? (
          <Text size="sm" c="dimmed" mt="xs">
            No text chunks (tEXt, zTXt, iTXt, COM) present in this image.
          </Text>
        ) : (
          chunks.map((chunk, idx) => (
            <div key={idx} className={classes.chunkBox}>
              <div className={classes.chunkHeader}>
                <Group gap="xs">
                  <span className={classes.chunkKey}>{chunk.key}</span>
                  <Badge size="xs" color="gray" variant="light">
                    {chunk.chunk_type}
                  </Badge>
                </Group>

                <CopyButton value={chunk.value}>
                  {({ copied, copy }) => (
                    <Tooltip label={copied ? 'Copied!' : 'Copy Value'}>
                      <ActionIcon
                        variant="subtle"
                        color={copied ? 'teal' : 'gray'}
                        size="xs"
                        onClick={copy}
                      >
                        {copied ? (
                          <IconCheck size={12} />
                        ) : (
                          <IconCopy size={12} />
                        )}
                      </ActionIcon>
                    </Tooltip>
                  )}
                </CopyButton>
              </div>

              <div className={classes.chunkValue}>{chunk.value}</div>
            </div>
          ))
        )}
      </div>
    </Stack>
  )
}
