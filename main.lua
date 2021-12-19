_G.RefuelCheck = function()
    fuelLevel = turtle.getFuelLevel()
    if fuelLevel < 500 then
        turtle.select(15)
        local amount = math.ceil(turtle.getItemCount(15) - 1)
        if amount > 0 then
            turtle.refuel(amount)
        end
    end
    print("Fuel level = ", fuelLevel)
    info("fuelLevel", fuelLevel)
    if fuelLevel < 200 then
        print("Low fuel level !")
        return 1
    end
    return 0
end

_G.depositItem = function()
    print("Start item deposit")
    while turtle.detectUp() do
        if not turtle.digUp() then
            forwardforceDig()
        end
    end
    turtle.select(16)
    if not turtle.detectUp() then
        if turtle.placeUp() then
            for i = 1, 14 do
                turtle.select(i)
                turtle.dropUp()
            end
            turtle.select(16)
            turtle.drop()
            turtle.digUp()
        end
    end
end

_G.Up = function(height)
    for i = 1, height do
        while turtle.detectUp() do
          if not turtle.digUp() then
            info("stuck", "up")
            sleep(60)
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
              sleep(60)
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

_G.UpDig = function(height)
    for i = 1, height do
      if turtle.detect() then
        turtle.dig()
      end
      _G.Up(1)
    end
end

_G.DownDig = function(height)
    for i = 1, height do
        while turtle.detect() do
            turtle.dig()
        end
        _G.Down(1)
    end
end

_G.Forward = function(x)
    for i = 1, x do
        while turtle.detect() do
            turtle.dig()
        end
        turtle.forward()
    end
end

_G.Sleep = sleep

_G.Reboot = function()
    shell.execute("reboot")
end

_G.goTop = function()
    while turtle.detect() do
        turtle.up()
    end
    turtle.forward()
end

api_url = "http://13cc-90-16-73-173.ngrok.io"
_G.info = function(topic, info)
    local request = http.post(api_url .. "/info/" .. turtlename .. '/' .. topic, "info=" .. tostring(info))
end

turtlename = os.getComputerLabel()
while 1 do
    _G.RefuelCheck()
    local request = http.get(api_url .. "/request/" .. turtlename)
    for line in request.readLine do
        print("received command from server:" .. line)
        if not pcall(loadstring(line)) then
            print("Coulnd't do: " .. line)
            break
        end
    end
    sleep(1)
end
