use asmov_else_model::{self as model, prelude::*, prelude::build::*};

pub struct ClientSession {
    interface_view: model::InterfaceView,
}

impl ClientSession {
    pub fn interface_view(&self) -> &model::InterfaceView {
        &self.interface_view
    }

    pub fn todo_from_universe_server(world: &model::World) -> model::Result<Self> {
        let mut interface_view_creator = model::InterfaceViewBuilder::creator();

        interface_view_creator.interface(model::testing::interface_from_universe())?;

        interface_view_creator.world_view({
            let mut world_view_creator = model::WorldViewBuilder::creator();
            world_view_creator.uid(world.uid())?;
            world_view_creator.frame(world.frame())?;

            let backyard_area = world.find_area(model::testing::BACKYARD)?; 

            world_view_creator.area_view({
                let mut area_view_creator = model::AreaViewBuilder::creator();
                area_view_creator.uid(backyard_area.uid())?;
                area_view_creator.descriptor(model::DescriptorBuilder::clone_model(model::BuilderMode::Creator, backyard_area.descriptor()))?;
                
                for occupant_uid in backyard_area.occupant_uids() {
                    area_view_creator.add_occupant_uid(*occupant_uid)?;
                }

                for route_uid in backyard_area.route_uids() {
                    area_view_creator.add_route_uid(*route_uid)?;
                }

                area_view_creator
            })?;

            for route_uid in backyard_area.route_uids() {
                let route = world.route(*route_uid)?;
                let route_builder = RouteBuilder::clone_model(model::BuilderMode::Creator, &route);
                world_view_creator.add_route(route_builder)?;
            } 

            for thing_uid in backyard_area.occupant_uids() {
                let thing = world.thing(*thing_uid)?;
                let mut thing_view_creator = model::ThingViewBuilder::creator();
                thing_view_creator.uid(thing.uid())?;
                thing_view_creator.descriptor(model::DescriptorBuilder::clone_model(model::BuilderMode::Creator, thing.entity().descriptor()))?;
                world_view_creator.add_thing_view(thing_view_creator)?;
            } 

            world_view_creator
        })?;

        interface_view_creator.downlink_uid({
            world
                .find_thing(model::testing::HOUSEKEEPER).unwrap()
                .uid()
        })?;

        let (_, interface_view) = interface_view_creator.create()?.split();

        Ok(Self {
            interface_view
        })
    }
}

