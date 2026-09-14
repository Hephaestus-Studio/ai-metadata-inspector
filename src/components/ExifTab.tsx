import React, { useState } from 'react'
import {
  Accordion,
  Anchor,
  Badge,
  Group,
  Stack,
  Table,
  Text,
  TextInput,
} from '@mantine/core'
import {
  IconCamera,
  IconExternalLink,
  IconList,
  IconMapPin,
  IconSearch,
} from '@tabler/icons-react'
import type { ExifReport, ImageMetadataReport } from '@/types/metadata'
import { TabEmptyState } from './TabEmptyState'
import classes from './ExifTab.module.css'

interface ExifTabProps {
  exif?: ExifReport
  report?: ImageMetadataReport
  onNavigateTab?: (tab: string) => void
}

export const ExifTab: React.FC<ExifTabProps> = ({
  exif,
  report,
  onNavigateTab,
}) => {
  const [tagSearch, setTagSearch] = useState('')

  if (!exif || exif.all_tags.length === 0) {
    return (
      <TabEmptyState
        icon={<IconCamera size={28} stroke={1.5} />}
        title="No EXIF or GPS Camera Metadata"
        description="Camera hardware, exposure settings (ISO, shutter, aperture), and GPS location tags were not found in this image."
        currentTab="exif"
        report={report}
        onNavigateTab={onNavigateTab}
      />
    )
  }

  const filteredTags = exif.all_tags.filter(
    (t) =>
      t.tag_name.toLowerCase().includes(tagSearch.toLowerCase()) ||
      t.value.toLowerCase().includes(tagSearch.toLowerCase()),
  )

  return (
    <Stack gap="md">
      {/* Overview Camera Hardware & Exposure Grid */}
      <div className={classes.grid}>
        {exif.camera_make && (
          <div className={classes.card}>
            <div className={classes.label}>Camera Make</div>
            <div className={classes.value}>{exif.camera_make}</div>
          </div>
        )}

        {exif.camera_model && (
          <div className={classes.card}>
            <div className={classes.label}>Camera Model</div>
            <div className={classes.value}>{exif.camera_model}</div>
          </div>
        )}

        {exif.lens_model && (
          <div className={classes.card} style={{ gridColumn: 'span 2' }}>
            <div className={classes.label}>Lens Model</div>
            <div className={classes.value}>{exif.lens_model}</div>
          </div>
        )}

        {exif.date_time && (
          <div className={classes.card}>
            <div className={classes.label}>Date & Time</div>
            <div className={classes.value}>{exif.date_time}</div>
          </div>
        )}

        {exif.iso && (
          <div className={classes.card}>
            <div className={classes.label}>ISO Speed</div>
            <div className={classes.value}>{exif.iso}</div>
          </div>
        )}

        {exif.f_number && (
          <div className={classes.card}>
            <div className={classes.label}>Aperture</div>
            <div className={classes.value}>{exif.f_number}</div>
          </div>
        )}

        {exif.exposure_time && (
          <div className={classes.card}>
            <div className={classes.label}>Shutter Speed</div>
            <div className={classes.value}>{exif.exposure_time}</div>
          </div>
        )}

        {exif.focal_length && (
          <div className={classes.card}>
            <div className={classes.label}>Focal Length</div>
            <div className={classes.value}>{exif.focal_length}</div>
          </div>
        )}

        {exif.software && (
          <div className={classes.card}>
            <div className={classes.label}>Software</div>
            <div className={classes.value}>{exif.software}</div>
          </div>
        )}

        {exif.serial_number && (
          <div className={classes.card}>
            <div className={classes.label} style={{ color: '#ef4444' }}>
              Device Serial
            </div>
            <div className={classes.value}>{exif.serial_number}</div>
          </div>
        )}
      </div>

      {/* GPS Geolocation Map Section */}
      {exif.gps && (
        <div style={{ marginTop: 10 }}>
          <Group justify="space-between" align="center" mb={6}>
            <Group gap="xs">
              <Badge
                color="red"
                variant="filled"
                size="md"
                leftSection={<IconMapPin size={14} />}
              >
                GPS Location Detected
              </Badge>
              <Text size="xs" fw={600}>
                {exif.gps.formatted_coords}
              </Text>
            </Group>

            <Group gap="xs">
              <Anchor
                href={exif.gps.google_maps_url}
                target="_blank"
                rel="noopener noreferrer"
                size="xs"
              >
                Google Maps <IconExternalLink size={12} />
              </Anchor>
              <Anchor
                href={exif.gps.osm_url}
                target="_blank"
                rel="noopener noreferrer"
                size="xs"
              >
                OpenStreetMap <IconExternalLink size={12} />
              </Anchor>
            </Group>
          </Group>

          {/* Embedded OpenStreetMap Iframe */}
          <div className={classes.mapContainer}>
            <iframe
              title="GPS Location Map"
              width="100%"
              height="100%"
              style={{ border: 0 }}
              loading="lazy"
              src={`https://www.openstreetmap.org/export/embed.html?bbox=${
                exif.gps.longitude - 0.01
              },${exif.gps.latitude - 0.01},${exif.gps.longitude + 0.01},${
                exif.gps.latitude + 0.01
              }&layer=mapnik&marker=${exif.gps.latitude},${exif.gps.longitude}`}
            />
          </div>
        </div>
      )}

      {/* All EXIF Tags Table Accordion */}
      {exif.all_tags.length > 0 && (
        <Accordion variant="separated" radius="md" mt="xs">
          <Accordion.Item value="all-tags">
            <Accordion.Control icon={<IconList size={18} color="#6366f1" />}>
              <Text size="sm" fw={600}>
                View All Extracted EXIF Tags ({exif.all_tags.length})
              </Text>
            </Accordion.Control>
            <Accordion.Panel>
              <TextInput
                placeholder="Search tag name or value..."
                size="xs"
                mb="sm"
                leftSection={<IconSearch size={14} />}
                value={tagSearch}
                onChange={(e) => setTagSearch(e.currentTarget.value)}
              />
              <div className={classes.tableWrapper}>
                <Table striped highlightOnHover>
                  <Table.Thead>
                    <Table.Tr>
                      <Table.Th>Tag Name</Table.Th>
                      <Table.Th>IFD</Table.Th>
                      <Table.Th>Value</Table.Th>
                    </Table.Tr>
                  </Table.Thead>
                  <Table.Tbody>
                    {filteredTags.map((tag, idx) => (
                      <Table.Tr key={idx}>
                        <Table.Td className={classes.tagName}>
                          {tag.tag_name}
                        </Table.Td>
                        <Table.Td className={classes.ifdBadge}>
                          {tag.ifd}
                        </Table.Td>
                        <Table.Td className={classes.tagValue}>
                          {tag.value}
                        </Table.Td>
                      </Table.Tr>
                    ))}
                  </Table.Tbody>
                </Table>
              </div>
            </Accordion.Panel>
          </Accordion.Item>
        </Accordion>
      )}
    </Stack>
  )
}
