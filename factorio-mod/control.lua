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
    
    return game.table_to_json(response)
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
        local recipe = game.recipe_prototypes[params.recipe]
        if recipe and entity.get_recipe() ~= recipe then
            entity.set_recipe(params.recipe)
        end
    end
    
    -- Set modules if provided
    if params.modules and entity.get_module_inventory() then
        local inventory = entity.get_module_inventory()
        for i, module_name in ipairs(params.modules) do
            if i <= inventory.getbar() then
                inventory.insert({name = module_name, count = 1})
            end
        end
    end
    
    return create_response(true, "place", {
        entity = params.entity,
        position = position,
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
        radius = 0.5,
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
    if entity.type == "assembling-machine" and entity.get_recipe() then
        entity_data.recipe = entity.get_recipe().name
    end
    
    -- Add modules if present
    if entity.get_module_inventory() then
        local inventory = entity.get_module_inventory()
        local modules = {}
        for i = 1, #inventory do
            local stack = inventory[i]
            if stack.valid_for_read then
                table.insert(modules, stack.name)
            end
        end
        if #modules > 0 then
            entity_data.modules = modules
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
        radius = 0.5,
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

-- Register the remote interface
remote.add_interface("fcb", fcb)

-- Log on init
script.on_init(function()
    game.print("Factorio Constraint Builder mod loaded")
    game.print("Remote interface 'fcb' registered for RCON control")
end)
