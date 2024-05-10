function damage_about_to_be_received( damage, x, y, entity_thats_responsible, critical_hit_chance )
     if(entity_thats_responsible ~= GameGetWorldStateEntity())then
         return 0, 0
     end

     return damage, 0
 end