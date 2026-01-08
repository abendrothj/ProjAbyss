#pragma once

#include "CoreMinimal.h"
#include "GameFramework/Pawn.h"
#include "OceanSolver.h" // We include our math struct here
#include "MasterShip.generated.h"

UCLASS()
class PROJABYSS_API AMasterShip : public APawn
{
    GENERATED_BODY()

public:
    AMasterShip();

protected:
    virtual void BeginPlay() override;

public:
    virtual void Tick(float DeltaTime) override;

    // -- COMPONENTS --

    // The physical body of the ship (The Hull)
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Ship")
    UStaticMeshComponent* HullMesh;

    // The 4 invisible "Floaters" at the corners
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Buoyancy")
    USceneComponent* PontoonFL; // Front Left
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Buoyancy")
    USceneComponent* PontoonFR; // Front Right
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Buoyancy")
    USceneComponent* PontoonBL; // Back Left
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Buoyancy")
    USceneComponent* PontoonBR; // Back Right

    // -- SETTINGS --

    // How strong the water pushes up. Tweak this in Editor!
    UPROPERTY(EditAnywhere, Category = "Buoyancy")
    float FloatForce = 800.0f;

    // Water damping (drag) to stop infinite bouncing
    UPROPERTY(EditAnywhere, Category = "Buoyancy")
    float WaterDrag = 20.0f;

private:
    // Our C++ Math Calculator (From the struct we made earlier)
    FOceanSolver OceanSolver;
};