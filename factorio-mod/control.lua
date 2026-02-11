-- Factorio Constraint Builder Mod
-- Main control file

-- Remote interface for RCON communication
local fcb = {}

-- Helper function to create JSON-like response
local function create_response(success, action, data)
    local response = {
        success = success,
        action = action
    }

    -- Merge data into response
    if data then
        for k, v in pairs(data) do
            response[k] = v
        end
    end

    -- Factorio 2.0 moved table_to_json to helpers
    return helpers.table_to_json(response)
end

-- Direction name to defines.direction mapping
local direction_map = {
    north = defines.direction.north,
    south = defines.direction.south,
    east = defines.direction.east,
    west = defines.direction.west,
    northeast = defines.direction.northeast,
    northwest = defines.direction.northwest,
    southeast = defines.direction.southeast,
    southwest = defines.direction.southwest,
    n = defines.direction.north,
    s = defines.direction.south,
    e = defines.direction.east,
    w = defines.direction.west,
    ne = defines.direction.northeast,
    nw = defines.direction.northwest,
    se = defines.direction.southeast,
    sw = defines.direction.southwest
}

-- Convert direction name or number to defines.direction
local function parse_direction(dir)
    if type(dir) == "number" then
        return dir
    elseif type(dir) == "string" then
        return direction_map[dir:lower()] or defines.direction.north
    end
    return defines.direction.north
end

-- Place entity command
-- Parameters: {entity, position, direction, recipe, modules}
function fcb.place(params)
    if not params.entity then
        return create_response(false, "place", {
            error = "missing_parameter",
            message = "Entity type is required"
        })
    end
    
    if not params.position or not params.position.x or not params.position.y then
        return create_response(false, "place", {
            error = "missing_parameter",
            message = "Position {x, y} is required"
        })
    end
    
    local surface = game.surfaces[1]  -- Get default surface
    local position = params.position
    local direction = parse_direction(params.direction or 0)
    
    -- Check if we can place the entity
    local can_place = surface.can_place_entity{
        name = params.entity,
        position = position,
        direction = direction,
        force = "player"
    }
    
    if not can_place then
        -- Find what's blocking
        local blocking = surface.find_entities_filtered{
            position = position,
            radius = 2,
            limit = 1
        }
        
        local blocking_info = nil
        if blocking and #blocking > 0 then
            blocking_info = {
                name = blocking[1].name,
                position = blocking[1].position
            }
        end
        
        return create_response(false, "place", {
            error = "blocked",
            message = "Cannot place entity at position",
            entity = params.entity,
            position = position,
            blocking_entity = blocking_info
        })
    end
    
    -- Place the entity
    local entity = surface.create_entity{
        name = params.entity,
        position = position,
        direction = direction,
        force = "player",
        raise_built = true
    }
    
    if not entity then
        return create_response(false, "place", {
            error = "creation_failed",
            message = "Failed to create entity",
            entity = params.entity,
            position = position
        })
    end
    
    -- Set recipe if provided and entity supports it
    if params.recipe and entity.type == "assembling-machine" then
        -- Factorio 2.0: set_recipe might return false if recipe is not valid
        local success = pcall(function()
            entity.set_recipe(params.recipe)
        end)
        if not success then
            -- Recipe setting failed, but don't fail the whole operation
        end
    end

    -- Set modules if provided
    if params.modules and entity.get_module_inventory then
        local inventory = entity.get_module_inventory()
        if inventory then
            for i, module_name in ipairs(params.modules) do
                -- Factorio 2.0: use # operator for inventory length
                if i <= #inventory then
                    inventory.insert({name = module_name, count = 1})
                end
            end
        end
    end

    -- Set splitter settings if this is a splitter entity
    if entity.type == "splitter" then
        -- Set input priority (which input side to prefer pulling from)
        if params.input_priority then
            entity.splitter_input_priority = params.input_priority
        end

        -- Set output priority (which output side to prefer sending to)
        if params.output_priority then
            entity.splitter_output_priority = params.output_priority
        end

        -- Set filter (item to route to the priority output side)
        if params.filter then
            local item_prototype = game.item_prototypes[params.filter]
            if item_prototype then
                entity.splitter_filter = item_prototype
            end
        end
    end

    return create_response(true, "place", {
        entity = params.entity,
        position = entity.position,  -- Return actual position, not requested position
        direction = direction,
        unit_number = entity.unit_number
    })
end

-- Query entity at position
-- Parameters: {position}
function fcb.query(params)
    if not params.position or not params.position.x or not params.position.y then
        return create_response(false, "query", {
            error = "missing_parameter",
            message = "Position {x, y} is required"
        })
    end

    local surface = game.surfaces[1]
    local entities = surface.find_entities_filtered{
        position = params.position,
        radius = 1.5,
        limit = 1
    }
    
    if not entities or #entities == 0 then
        return create_response(true, "query", {
            position = params.position,
            entity = nil
        })
    end
    
    local entity = entities[1]
    local entity_data = {
        name = entity.name,
        position = entity.position,
        direction = entity.direction,
        type = entity.type
    }
    
    -- Add recipe if it's an assembling machine
    if entity.type == "assembling-machine" then
        -- Factorio 2.0: get_recipe might return nil
        local recipe = entity.get_recipe and entity.get_recipe()
        if recipe then
            entity_data.recipe = recipe.name
        end
    end

    -- Add modules if present
    if entity.get_module_inventory then
        local inventory = entity.get_module_inventory()
        if inventory then
            local modules = {}
            for i = 1, #inventory do
                local stack = inventory[i]
                if stack and stack.valid_for_read then
                    table.insert(modules, stack.name)
                end
            end
            if #modules > 0 then
                entity_data.modules = modules
            end
        end
    end

    -- Add splitter settings if this is a splitter
    if entity.type == "splitter" then
        entity_data.input_priority = entity.splitter_input_priority
        entity_data.output_priority = entity.splitter_output_priority
        if entity.splitter_filter then
            entity_data.filter = entity.splitter_filter.name
        end
    end

    return create_response(true, "query", {
        position = params.position,
        entity = entity_data
    })
end

-- Check if entity can be placed
-- Parameters: {entity, position}
function fcb.can_place(params)
    if not params.entity then
        return create_response(false, "can_place", {
            error = "missing_parameter",
            message = "Entity type is required"
        })
    end
    
    if not params.position or not params.position.x or not params.position.y then
        return create_response(false, "can_place", {
            error = "missing_parameter",
            message = "Position {x, y} is required"
        })
    end
    
    local surface = game.surfaces[1]
    local can_place = surface.can_place_entity{
        name = params.entity,
        position = params.position,
        direction = parse_direction(params.direction or 0),
        force = "player"
    }
    
    local reason = nil
    if not can_place then
        -- Try to determine why
        local blocking = surface.find_entities_filtered{
            position = params.position,
            radius = 2,
            limit = 1
        }
        
        if blocking and #blocking > 0 then
            reason = "blocked by " .. blocking[1].name
        else
            reason = "invalid position or entity"
        end
    end
    
    return create_response(true, "can_place", {
        entity = params.entity,
        position = params.position,
        can_place = can_place,
        reason = reason
    })
end

-- Clear area
-- Parameters: {area} where area = {left_top = {x, y}, right_bottom = {x, y}}
function fcb.clear(params)
    if not params.area then
        return create_response(false, "clear", {
            error = "missing_parameter",
            message = "Area {left_top, right_bottom} is required"
        })
    end
    
    local surface = game.surfaces[1]
    local entities = surface.find_entities_filtered{
        area = params.area,
        force = "player"
    }
    
    local cleared_count = 0
    for _, entity in pairs(entities) do
        if entity.valid and entity.minable then
            entity.destroy()
            cleared_count = cleared_count + 1
        end
    end
    
    return create_response(true, "clear", {
        area = params.area,
        cleared_count = cleared_count
    })
end

-- Remove entity at position
-- Parameters: {position}
function fcb.remove(params)
    if not params.position or not params.position.x or not params.position.y then
        return create_response(false, "remove", {
            error = "missing_parameter",
            message = "Position {x, y} is required"
        })
    end

    local surface = game.surfaces[1]
    local entities = surface.find_entities_filtered{
        position = params.position,
        radius = 1.5,
        limit = 1
    }
    
    if not entities or #entities == 0 then
        return create_response(false, "remove", {
            position = params.position,
            error = "not_found",
            message = "No entity found at position"
        })
    end
    
    local entity = entities[1]
    local entity_name = entity.name
    entity.destroy()
    
    return create_response(true, "remove", {
        position = params.position,
        entity = entity_name
    })
end

-- List entities in area
-- Parameters: {area}
function fcb.list_area(params)
    if not params.area then
        return create_response(false, "list_area", {
            error = "missing_parameter",
            message = "Area {left_top, right_bottom} is required"
        })
    end
    
    local surface = game.surfaces[1]
    local entities = surface.find_entities_filtered{
        area = params.area,
        force = "player"
    }
    
    local entity_list = {}
    for _, entity in pairs(entities) do
        table.insert(entity_list, {
            name = entity.name,
            position = entity.position,
            direction = entity.direction,
            type = entity.type
        })
    end
    
    return create_response(true, "list_area", {
        area = params.area,
        count = #entity_list,
        entities = entity_list
    })
end

-- Create blueprint from layout
-- Parameters: {name, entities}
-- entities = list of {entity_name, position, direction, recipe}
function fcb.create_blueprint(params)
    if not params.name then
        return create_response(false, "create_blueprint", {
            error = "missing_parameter",
            message = "Blueprint name is required"
        })
    end

    if not params.entities or #params.entities == 0 then
        return create_response(false, "create_blueprint", {
            error = "missing_parameter",
            message = "Entities list is required"
        })
    end

    local player = game.players[1]
    if not player then
        return create_response(false, "create_blueprint", {
            error = "no_player",
            message = "No player found"
        })
    end

    -- Create blueprint in player's inventory
    local blueprint = player.cursor_stack
    if not blueprint or not blueprint.valid_for_read then
        -- Get an empty slot in main inventory
        local inventory = player.get_main_inventory()
        local slot = inventory.find_empty_stack()
        if slot then
            blueprint = inventory[slot]
        else
            return create_response(false, "create_blueprint", {
                error = "no_space",
                message = "No space in inventory for blueprint"
            })
        end
    end

    blueprint.set_stack({name = "blueprint"})

    -- Build entity array for blueprint
    local bp_entities = {}
    for i, entity_data in ipairs(params.entities) do
        local bp_entity = {
            entity_number = i,
            name = entity_data.entity_name,
            position = entity_data.position or {x = 0, y = 0}
        }

        if entity_data.direction then
            bp_entity.direction = entity_data.direction
        end

        if entity_data.recipe then
            bp_entity.recipe = entity_data.recipe
        end

        table.insert(bp_entities, bp_entity)
    end

    -- Set blueprint contents
    blueprint.set_blueprint_entities(bp_entities)
    blueprint.label = params.name

    return create_response(true, "create_blueprint", {
        name = params.name,
        entity_count = #bp_entities
    })
end

-- Spawn chest with items near player
-- Parameters: {items = [{name, count}, ...]}
function fcb.spawn_chest(params)
    if not params.items or #params.items == 0 then
        return create_response(false, "spawn_chest", {
            error = "missing_parameter",
            message = "Items list is required"
        })
    end

    local player = game.players[1]
    if not player then
        return create_response(false, "spawn_chest", {
            error = "no_player",
            message = "No player found"
        })
    end

    local surface = game.surfaces[1]

    -- Find position near player
    local chest_pos = {x = player.position.x + 2, y = player.position.y}

    -- Create chest
    local chest = surface.create_entity{
        name = "iron-chest",
        position = chest_pos,
        force = "player",
        raise_built = true
    }

    if not chest then
        return create_response(false, "spawn_chest", {
            error = "placement_failed",
            message = "Could not place chest"
        })
    end

    -- Fill chest with items
    local inventory = chest.get_inventory(defines.inventory.chest)
    local items_added = {}

    for _, item_data in ipairs(params.items) do
        local inserted = inventory.insert({
            name = item_data.name,
            count = item_data.count
        })
        table.insert(items_added, {
            name = item_data.name,
            requested = item_data.count,
            inserted = inserted
        })
    end

    return create_response(true, "spawn_chest", {
        position = chest.position,
        items = items_added
    })
end

-- Get player position
function fcb.player_pos(params)
    local player = game.players[1]
    if not player then
        return create_response(false, "player_pos", {
            error = "no_player",
            message = "No player found"
        })
    end

    return create_response(true, "player_pos", {
        position = player.position
    })
end

-- Register the remote interface
remote.add_interface("fcb", fcb)

-- Log on init
script.on_init(function()
    game.print("Factorio Constraint Builder mod loaded")
    game.print("Remote interface 'fcb' registered for RCON control")
end)
