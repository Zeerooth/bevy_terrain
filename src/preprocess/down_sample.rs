use crate::preprocess::AveragePixel;
use crate::{
    preprocess::{
        file_io::{format_node_path, load_node, load_or_create_node, save_node},
        UVec2Utils,
    },
    skip_none,
    terrain_data::{AttachmentConfig, AttachmentFormat},
};
use bevy::prelude::*;
use image::{DynamicImage, Luma, LumaA, Rgb, Rgba};
use itertools::iproduct;

type Filter = fn(&mut DynamicImage, &DynamicImage, &AttachmentConfig, UVec2);

pub(crate) fn linear(
    node_image: &mut DynamicImage,
    child_image: &DynamicImage,
    attachment: &AttachmentConfig,
    offset: UVec2,
) {
    let child_size = attachment.center_size >> 1;
    let node_x = offset.x * child_size + attachment.border_size;
    let node_y = offset.y * child_size + attachment.border_size;

    match attachment.format {
        AttachmentFormat::Rgb8 => {
            let node_image = node_image.as_mut_rgb8().unwrap();
            let child_image = child_image.as_rgb8().unwrap();

            for (x, y) in iproduct!(0..child_size, 0..child_size) {
                let mut values = [[0; 3]; 4];
                for (i, value) in values.iter_mut().enumerate() {
                    *value = child_image
                        .get_pixel(
                            (x << 1) + attachment.border_size + (i as u32 >> 1),
                            (y << 1) + attachment.border_size + (i as u32 & 1),
                        )
                        .0
                }

                let value = AveragePixel::average(values[0], values[1], values[2], values[3]);

                node_image.put_pixel(node_x + x, node_y + y, Rgb(value));
            }
        }
        AttachmentFormat::Rgba8 => {
            let node_image = node_image.as_mut_rgba8().unwrap();
            let child_image = child_image.as_rgba8().unwrap();

            for (x, y) in iproduct!(0..child_size, 0..child_size) {
                let mut values = [[0; 4]; 4];
                for (i, value) in values.iter_mut().enumerate() {
                    *value = child_image
                        .get_pixel(
                            (x << 1) + attachment.border_size + (i as u32 >> 1),
                            (y << 1) + attachment.border_size + (i as u32 & 1),
                        )
                        .0
                }

                let value = AveragePixel::average(values[0], values[1], values[2], values[3]);

                node_image.put_pixel(node_x + x, node_y + y, Rgba(value));
            }
        }
        AttachmentFormat::R16 => {
            let node_image = node_image.as_mut_luma16().unwrap();
            let child_image = child_image.as_luma16().unwrap();

            for (x, y) in iproduct!(0..child_size, 0..child_size) {
                let mut values = [[0; 1]; 4];
                for (i, value) in values.iter_mut().enumerate() {
                    *value = child_image
                        .get_pixel(
                            (x << 1) + attachment.border_size + (i as u32 >> 1),
                            (y << 1) + attachment.border_size + (i as u32 & 1),
                        )
                        .0
                }

                let value = AveragePixel::average(values[0], values[1], values[2], values[3]);

                node_image.put_pixel(node_x + x, node_y + y, Luma(value));
            }
        }
        AttachmentFormat::Rg16 => {
            let node_image = node_image.as_mut_luma_alpha16().unwrap();
            let child_image = child_image.as_luma_alpha16().unwrap();

            for (x, y) in iproduct!(0..child_size, 0..child_size) {
                let mut values = [[0; 2]; 4];
                for (i, value) in values.iter_mut().enumerate() {
                    *value = child_image
                        .get_pixel(
                            (x << 1) + attachment.border_size + (i as u32 >> 1),
                            (y << 1) + attachment.border_size + (i as u32 & 1),
                        )
                        .0
                }

                let value = AveragePixel::average(values[0], values[1], values[2], values[3]);

                node_image.put_pixel(node_x + x, node_y + y, LumaA(value));
            }
        }
    }
}

pub(crate) fn minmax(
    node_image: &mut DynamicImage,
    child_image: &DynamicImage,
    attachment: &AttachmentConfig,
    offset: UVec2,
) {
    let node_image = node_image.as_mut_luma_alpha16().unwrap();
    let child_image = child_image.as_luma_alpha16().unwrap();

    let child_size = attachment.center_size >> 1;

    let node_x = offset.x * child_size + attachment.border_size;
    let node_y = offset.y * child_size + attachment.border_size;

    for (x, y) in iproduct!(0..child_size, 0..child_size) {
        let mut min = u16::MAX;
        let mut max = u16::MIN;

        for (cx, cy) in iproduct!(0..2, 0..2) {
            let value = child_image
                .get_pixel(
                    (x << 1) + cx + attachment.border_size,
                    (y << 1) + cy + attachment.border_size,
                )
                .0;
            min = min.min(value[0]);
            max = max.max(value[1]);
        }

        let value = LumaA([min, max]);
        node_image.put_pixel(node_x + x, node_y + y, value);
    }
}

pub(crate) fn down_sample_layer(
    filter: Filter,
    directory: &str,
    attachment: &AttachmentConfig,
    lod: u32,
    first: UVec2,
    last: UVec2,
) {
    for (x, y) in first.product(last) {
        let node_path = format_node_path(directory, attachment, lod, x, y);
        let mut node_image = load_or_create_node(&node_path, attachment);

        for (cx, cy) in iproduct!(0..2, 0..2) {
            let child_path =
                format_node_path(directory, attachment, lod - 1, (x << 1) + cx, (y << 1) + cy);
            let child_image = skip_none!(load_node(&child_path, attachment));
            // Todo: if a child node is not available, we should fill the gap in the parent one
            // maybe this should not even be possible

            filter(
                &mut node_image,
                &child_image,
                attachment,
                UVec2::new(cx, cy),
            );
        }

        save_node(&node_path, &node_image, attachment);
    }
}
