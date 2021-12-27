_G.RefuelCheck = function()
    local fuelLevel = turtle.getFuelLevel()
    if fuelLevel < 500 then
        turtle.select(16)
        local amount = math.ceil(turtle.getItemCount(16) - 1)
        if amount > 0 then
            turtle.refuel(amount)
        end
    end
    print("Fuel level = ", fuelLevel)
    return fuelLevel
end

_G.isFull = function ()
  local isFull = true
  for i = 1, 15 do
    turtle.select(i)
    if turtle.getItemCount() == 0 then
      isFull = false
      break
    end
  end
  return isFull
end

chestName = "minecraft:chest"
_G.DepositItem = function(filter)
    local has_block, inspect = turtle.inspect()
    if has_block and inspect.name == chestName then
      print("Start item deposit")
      turtle.inspect()
      for i = 1, 15 do
          turtle.select(i)
          while turtle.getItemCount() > 0 do
            turtle.drop()
          end
      end
    else
      info("issue", "missing chest")
      print("Missing chest")
    end
end

_G.Up = function(height)
    for i = 1, height do
        while turtle.detectUp() do
          if not turtle.digUp() then
            info("stuck", "up")
            break
          end
        end
        turtle.up()
    end
end

_G.Down = function(height)
    for i = 1, height do
        while turtle.detectDown() do
            if not turtle.digDown() then
              info("stuck", "down")
              break
            end
        end
        turtle.down()
    end
end

_G.Left = function(n)
  for i = 1, n do
    turtle.turnLeft()
  end
end

_G.Right = function(n)
  for i = 1, n do
    turtle.turnRight()
  end
end

-- _G.UpDig = function(height)
--     for i = 1, height do
--       if turtle.detect() then
--         turtle.dig()
--       end
--       _G.Up(1)
--     end
-- end

-- _G.DownDig = function(height)
--     for i = 1, height do
--         while turtle.detect() do
--             turtle.dig()
--         end
--         _G.Down(1)
--     end
-- end

_G.Forward = function(x)
    for i = 1, x do
        while turtle.detect() do
            if not turtle.dig() then
              info("stuck", "forward")
              break
            end
        end
        turtle.forward()
    end
end

_G.ForwardDig = function(height)
  for i = 1, height do
      while turtle.detectUp() do
          if not turtle.digUp() then
            break
          end
      end
      while turtle.detectDown() do
        if not turtle.digDown() then
          break
        end
      end
      _G.Forward(1)
  end
end


_G.Sleep = sleep

_G.Reboot = function()
    shell.execute("reboot")
end



api_url = "http://e61e-2001-861-3f0a-ec00-6870-79a6-b4e0-d8cd.ngrok.io"
_G.info = function(topic, info)
    local request = http.post(api_url .. "/info/" .. turtlename .. '/' .. topic, "info=" .. tostring(info))
    return request
end


turtlename = os.getComputerLabel()
while 1 do
    info("fuellevel", _G.RefuelCheck())
    info("isFull", _G.isFull())
    local request = http.get(api_url .. "/request/" .. turtlename)
    for line in request.readLine do
        print("Command from server:" .. line)
        if not pcall(loadstring(line)) then
            print("Coulnd't do: " .. line)
            break
        end
    end
    sleep(2)
end


-- _G.goTop = function()
--   while turtle.detect() do
--       turtle.up()
--   end
--   turtle.forward()
-- end

-- _G.enderChestDeposeItems = function()
--     print("Start item deposit")
--     while turtle.detectUp() do
--         if not turtle.digUp() then
--             forwardforceDig()
--         end
--     end
--     turtle.select(16)
--     if not turtle.detectUp() then
--         if turtle.placeUp() then
--             for i = 1, 14 do
--                 turtle.select(i)
--                 turtle.dropUp()
--             end
--             turtle.select(16)
--             turtle.drop()
--             turtle.digUp()
--         end
--     end
-- end