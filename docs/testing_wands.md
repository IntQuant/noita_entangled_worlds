{{Wand
| wandPic     = Wand handgun.png
| capacity    = 2
| spell1      = Teleport Bolt
}}

{{Wand Card
| wandPic      = Wand handgun.png
| castDelay    = 0.03
| rechargeTime = 0.15
| manaMax      = 1400.00
| manaCharge   = 310.00
| capacity     = 3
| spread       = -30
| speed        = 1.00
| spell1       = Black Hole
| spell2       = Black Hole
| spell3       = Black Hole
}}


{{Wand Card
| wandPic      = Wand handgun.png
| castDelay    = -10.0
| rechargeTime = -10.0
| manaMax      = 30000.00
| manaCharge   = 30000.00
| capacity     = 10
| spread       = -30
| speed        = 1.00
| spell1       = Spark bolt
}}


local x, y = cursor.pos()
for i = 1, 150 do
  EntityLoad("data/entities/props/physics_box_explosive.xml", x, y)
end